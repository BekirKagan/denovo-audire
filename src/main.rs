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
    thread,
    time::Duration,
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
    let mut _queue: Vec<Music> = Vec::new();
    let mut current_music: Music;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    render_musics(&mut stdout, &musics)?;

    while running {
        thread::sleep(Duration::from_millis(16)); // 60 frames per second

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
                        current_music = musics[index].clone();
                        play_music_once(&sink, &current_music)?;
                    }
                    _ => {}
                }
                stdout.flush()?;
            }
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
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(cursor::MoveToColumn(0))?;
        counter += 1;
    }
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn get_current_index(offset: u16) -> Result<usize, Error> {
    let index = cursor::position()?.1 - offset;
    Ok(index as usize)
}

fn play_music_once(sink: &Sink, music: &Music) -> Result<(), Error> {
    let source = get_source(&music.path)?;
    sink.stop();
    sink.append(source);
    Ok(())
}
