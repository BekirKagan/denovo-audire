use crossterm::{
    cursor, event,
    style::{self, Stylize},
    terminal, QueueableCommand,
};
use dirs_next::audio_dir;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    fs::File,
    io::{stdout, BufReader, Error, Stdout, Write},
    path::PathBuf,
};

mod music;
use music::Music;

fn main() -> Result<(), Error> {
    let mut stdout = stdout();
    let mut running = true;

    let offset_from_top: u16 = 2;

    // Initialize
    terminal::enable_raw_mode()?;
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, offset_from_top))?;
    stdout.queue(cursor::SetCursorStyle::SteadyBlock)?;
    stdout.flush()?;

    let musics_directory = audio_dir().expect("Could not find 'Musics' folder in user directory.");
    let musics_directory = musics_directory.to_str().unwrap();
    let musics: Vec<Music> = get_musics(musics_directory)?;
    let mut queue: Vec<Music> = Vec::new();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    render_musics(&mut stdout, &musics)?;
    render_queue(&mut stdout, &queue)?;
    render_volume(&mut stdout, &sink)?;

    while running {
        if let event::Event::Key(event) = event::read()? {
            if event.kind == event::KeyEventKind::Press {
                match event.code {
                    event::KeyCode::Esc => running = false,
                    event::KeyCode::Up | event::KeyCode::Char('k') => {
                        let prev_row = cursor::position()?.1 - 1;
                        if prev_row != offset_from_top - 1 {
                            stdout.queue(cursor::MoveToPreviousLine(1))?;
                        }
                    }
                    event::KeyCode::Down | event::KeyCode::Char('j') => {
                        let next_row = cursor::position()?.1 + 1;
                        if next_row != musics.len() as u16 + offset_from_top {
                            stdout.queue(cursor::MoveToNextLine(1))?;
                        }
                    }
                    event::KeyCode::Enter => {
                        let index = get_current_index(offset_from_top)?;
                        let music = musics[index].clone();
                        clear_queue(&mut stdout, &sink, &mut queue)?;
                        queue.push(music.clone());
                        play_queue(&sink, &queue)?;
                        render_queue(&mut stdout, &queue)?;
                        render_now_playing(&mut stdout, music.name)?;
                    }
                    event::KeyCode::Char(' ') => {
                        toggle_music(&sink)?;
                    }
                    event::KeyCode::Char('q') => {
                        let index = get_current_index(offset_from_top)?;
                        let music = musics[index].clone();
                        queue.push(music);
                        render_queue(&mut stdout, &queue)?;
                    }
                    event::KeyCode::Char('x') => {
                        clear_queue(&mut stdout, &sink, &mut queue)?;
                    }
                    event::KeyCode::Char('p') => {
                        sink.stop();
                        play_queue(&sink, &queue)?;
                        render_now_playing(&mut stdout, queue[0].name.clone())?;
                    }
                    event::KeyCode::Right | event::KeyCode::Char('l') => {
                        adjust_volume(&sink, 0.1)?;
                        render_volume(&mut stdout, &sink)?;
                    }
                    event::KeyCode::Left | event::KeyCode::Char('h') => {
                        adjust_volume(&sink, -0.1)?;
                        render_volume(&mut stdout, &sink)?;
                    }
                    _ => {}
                }
            }
            stdout.flush()?;
        }
    }

    // Shutdown
    terminal::disable_raw_mode()?;
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(cursor::SetCursorStyle::DefaultUserShape)?;
    stdout.queue(style::ResetColor)?;
    stdout.flush()?;

    Ok(())
}

fn get_source(file_path: &PathBuf) -> Result<Decoder<BufReader<File>>, Error> {
    let path = file_path.to_str().unwrap();
    let file = File::open(&path).expect(format!("Couldn't open file: {path}").as_str());
    let reader = BufReader::new(file);
    let source = Decoder::new(reader).expect(format!("Couldn't decode file: {path}").as_str());
    Ok(source)
}

fn get_musics(musics_path: &str) -> Result<Vec<Music>, Error> {
    let mut musics: Vec<Music> = Vec::new();
    let supported_extensions = ["mp3"];
    for entry in std::fs::read_dir(musics_path)? {
        let entry = entry?;
        let file_path = entry.path();
        let file_name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
        let file_extension = file_path.extension().unwrap_or_default().to_str().unwrap();
        if supported_extensions.contains(&file_extension) {
            let source = get_source(&file_path)?;
            let file_duration = source
                .total_duration()
                .expect(format!("Could not get the total duration of file: {file_name}").as_str())
                .as_secs_f32();
            let music = Music {
                name: file_name,
                path: file_path,
                duration: file_duration,
            };
            musics.push(music);
        }
    }
    Ok(musics)
}

fn render_musics(stdout: &mut Stdout, musics: &Vec<Music>) -> Result<(), Error> {
    let terminal_width: usize = terminal::size()?.0 as usize;
    let mut counter: u16 = 0;
    stdout.queue(cursor::SavePosition)?;
    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.write("#".blue().to_string().as_bytes())?;
    stdout.queue(cursor::MoveToColumn(3))?;
    stdout.write("Name".blue().to_string().as_bytes())?;
    stdout.queue(cursor::MoveToColumn(40))?;
    stdout.write("Time".blue().to_string().as_bytes())?;
    stdout.queue(cursor::MoveToColumn(91))?;
    stdout.write("Queue".blue().to_string().as_bytes())?;
    stdout.queue(cursor::MoveTo(0, 1))?;
    stdout.write("─".repeat(terminal_width).blue().to_string().as_bytes())?;
    stdout.queue(cursor::MoveTo(0, 2))?;
    for music in musics {
        stdout.write(counter.to_string().as_bytes())?;
        stdout.queue(cursor::MoveToColumn(3))?;
        stdout.write(music.name.as_bytes())?;
        stdout.queue(cursor::MoveToColumn(40))?;
        stdout.write(music.duration.to_string().as_bytes())?;
        stdout.write(b"s")?;
        stdout.queue(cursor::MoveToColumn(89))?;
        stdout.write("│".blue().to_string().as_bytes())?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(cursor::MoveToColumn(0))?;
        counter += 1;
    }
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn render_now_playing(stdout: &mut Stdout, music_name: String) -> Result<(), Error> {
    let height = terminal::size()?.1;
    stdout.queue(cursor::SavePosition)?;
    stdout.queue(cursor::MoveTo(0, height - 2))?;
    stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
    stdout.write(
        format!("Now playing: {}", music_name)
            .blue()
            .to_string()
            .as_bytes(),
    )?;
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn render_queue(stdout: &mut Stdout, queue: &Vec<Music>) -> Result<(), Error> {
    stdout.queue(cursor::SavePosition)?;
    stdout.queue(cursor::MoveTo(91, 2))?;
    for music in queue {
        stdout.write(music.name.as_bytes())?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(cursor::MoveToColumn(91))?;
    }
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn clear_queue(stdout: &mut Stdout, sink: &Sink, queue: &mut Vec<Music>) -> Result<(), Error> {
    stdout.queue(cursor::SavePosition)?;
    stdout.queue(cursor::MoveTo(91, 2))?;
    for _ in &mut *queue {
        stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(cursor::MoveToColumn(91))?;
    }
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    queue.clear();
    sink.stop();
    Ok(())
}

fn render_volume(stdout: &mut Stdout, sink: &Sink) -> Result<(), Error> {
    let (width, height) = terminal::size()?;
    let current_volume = sink.volume();
    let start_position = width - 11;
    let cell_count = (current_volume * 10.0).round() as u16;
    stdout.queue(cursor::SavePosition)?;
    stdout.queue(cursor::MoveTo(start_position, height - 1))?;
    stdout.queue(terminal::Clear(terminal::ClearType::UntilNewLine))?;
    for index in 0..cell_count {
        stdout.queue(cursor::MoveToColumn(start_position + index))?;
        stdout.write("■".blue().to_string().as_bytes())?;
    }
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn adjust_volume(sink: &Sink, value: f32) -> Result<(), Error> {
    let new_volume = (sink.volume() + value).clamp(0.0, 1.0);
    sink.set_volume(new_volume);
    Ok(())
}

fn get_current_index(offset: u16) -> Result<usize, Error> {
    let index = cursor::position()?.1 - offset;
    Ok(index as usize)
}

fn toggle_music(sink: &Sink) -> Result<(), Error> {
    if sink.is_paused() {
        sink.play();
    } else {
        sink.pause();
    }
    Ok(())
}

fn play_queue(sink: &Sink, queue: &Vec<Music>) -> Result<(), Error> {
    for music in queue {
        let source = get_source(&music.path)?;
        sink.append(source);
    }
    Ok(())
}
