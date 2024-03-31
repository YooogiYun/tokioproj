use bytes::buf;
use std::{
    env,
    os::fd::AsFd,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
};

macro_rules! println_fn_location {
    () => {
        let location = std::panic::Location::caller();
        println!("Start of {:?}", location);
    };
}

fn println_fn_name<F>(_: F) {
    println!("start of {}", std::any::type_name::<F>());
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Start of Main");
    let abs_file_in_path = get_abs_path("./assets/data/foo.txt").await?;
    let abs_file_out_path = get_abs_path("./assets/data/bar.txt").await?;

    let mut f_in = File::open(&abs_file_in_path).await?;
    read_content_within(&mut f_in, 10).await?;
    read_all_content(&mut f_in).await?;

    let mut f_out = File::create(&abs_file_out_path).await?;
    write_content_to(&mut f_out, "I'm the writer who writes bytes into a file.\n").await?;
    write_all_to(
        &mut f_out,
        "I'm the second writer who writes bytes into a file.\n",
    )
    .await?;

    // copy_all_to(&mut f_in).await?; // It's not allowed due to the f_in is hold by a reader which is read-only
    copy_all_to(
        &mut f_out,
        "I'm the third writer who writes bytes into a file by the method of Copying.\n",
    )
    .await?;

    Ok(())
}

async fn get_abs_path(relative_path: &str) -> io::Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let abs_file_path = PathBuf::from(current_dir).join(relative_path);
    println!("The abs file path is: {}", abs_file_path.display());
    Ok(abs_file_path)
}

async fn read_content_within(file: &mut File, length: usize) -> io::Result<()> {
    // let mut buffer = [0; length as usize];
    // println!("start of read_all_content");
    println_fn_name(read_content_within);
    let mut buffer = vec![0; length];
    let n = file.read(&mut buffer).await?;
    println!("GOT FROM FILE: {:?}", &buffer[..n]);
    Ok(())
}

async fn read_all_content(file: &mut File) -> io::Result<()> {
    // println!("start of read_all_content");
    println_fn_name(read_all_content);
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    println!("GOT FROM FILE: {:?}", &buffer[..buffer.len()]);
    Ok(())
}

async fn write_content_to(file: &mut File, content: &str) -> io::Result<()> {
    println_fn_name(write_content_to);
    let n = file.write(content.as_bytes()).await?;
    println!(
        "Total {} bytes, but only {} bytes have been written into {:?}",
        content.as_bytes().len(),
        n,
        file
    );
    Ok(())
}

async fn write_all_to(file: &mut File, content: &str) -> io::Result<()> {
    println_fn_name(write_all_to);
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

async fn copy_all_to(file: &mut File, content: &str) -> io::Result<()> {
    println_fn_name(copy_all_to);
    let mut buffer = content.as_bytes();
    io::copy(&mut buffer, file).await?;
    Ok(())
}
