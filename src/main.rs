use std::io::{self, Write}; 
use std::sync::{Arc, Mutex}; 
use std::thread; 
use reqwest::Url; 
use bytes::Bytes; 
 
fn main() -> Result<(), Box<dyn std::error::Error>> { 
    // Получаем URL файла из аргументов командной строки 
    let args: Vec<String> = std::env::args().collect(); 
    if args.len() != 2 { 
        eprintln!("Usage: cargo run --release -- <file_url>"); 
        return Ok(()); 
    } 
    let file_url = &args[1]; 
 
    // Создаем счетчик принятых байтов и оборачиваем его в Arc<Mutex<_>> для синхронизации доступа из нескольких потоков 
    let bytes_counter = Arc::new(Mutex::new(0u64)); 
 
    // Клонируем счетчик для передачи в поток вывода прогресса 
    let progress_counter = Arc::clone(&bytes_counter); 
 
    // Запускаем поток вывода прогресса 
    thread::spawn(move || { 
        loop { 
            // Ждем 1 секунду 
            thread::sleep(std::time::Duration::from_secs(1)); 
 
            // Получаем значение счетчика и выводим его 
            let count = *progress_counter.lock().unwrap(); 
            println!("Принято байтов: {}", count); 
        } 
    }); 
 
    // Создаем HTTP-клиент 
    let client = reqwest::blocking::Client::new(); 
 
    // Отправляем GET-запрос и получаем поток данных 
    let response = client.get(Url::parse(file_url)?).send()?; 
    let url_clone = response.url().clone(); // Клонируем URL до перемещения response в response_body 
    let mut response = response.error_for_status()?; 
    let mut response_body = response.bytes()?; 
 
    // Создаем файл для записи данных 
    let file_name = url_clone 
        .path_segments() 
        .and_then(|segments| segments.last()) 
        .and_then(|name| if name.is_empty() { None } else { Some(name) }) 
        .unwrap_or("downloaded_file"); 
    let mut file = std::fs::File::create(file_name)?; 
 
    // Создаем TeeReader для разделения потока данных на два: для записи в файл и для обновления счетчика принятых байтов 
    let mut tee_reader = std::io::copy(&mut response_body.as_ref(), &mut io::sink())?; 
 
// Читаем данные из response_body и обновляем счетчик принятых байтов 
let mut buffer = [0; 8192]; 
loop { 
    let read = { 
        let chunk = response_body.as_ref(); 
        let len = chunk.len().min(buffer.len()); 
        buffer[..len].copy_from_slice(&chunk[..len]); 
        len 
    }; 
    if read == 0 { 
        break; 
    } 
    file.write_all(&buffer[..read])?; 
    *bytes_counter.lock().unwrap() += read as u64; 
} 
 
    // Все завершено успешно 
    Ok(()) 
}