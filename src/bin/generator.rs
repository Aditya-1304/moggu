use std::io::{self, Write};
fn main() -> io::Result<()> {
  
    let width = 256;
    let height = 256;

    let mut stdout = io::stdout();
    
    writeln!(stdout, "P6")?;
    writeln!(stdout, "{} {}", width, height)?;
    writeln!(stdout, "255")?;
    for j in 0..height {
        for i in 0..width {
            let r = i as u8;
            let g = j as u8;
            let b = 128 as u8;

            let pixel = [r, g, b];

            stdout.write_all(&pixel)?;
        }
    }

    Ok(())
}