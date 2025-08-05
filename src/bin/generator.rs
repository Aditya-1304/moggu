use std::io::{self, Write};

// This program's purpose is to generate a PPM image and print its bytes
// to standard output. We can then redirect this output to a file to create
// the image.

fn main() -> io::Result<()> {
    // --- Image Properties ---
    // Let's create a reasonably sized image, 256 pixels wide and 256 pixels high.
    let width = 256;
    let height = 256;

    // --- PPM Header ---
    // We write the header directly to standard output.
    // `stdout()` gives us a handle to the standard output stream of the process.
    let mut stdout = io::stdout();
    
    // Write the magic number "P6", a newline, the width and height, another newline,
    // and the maximum color value (255), followed by one last newline.
    // This is the exact format our parser expects.
    writeln!(stdout, "P6")?;
    writeln!(stdout, "{} {}", width, height)?;
    writeln!(stdout, "255")?;

    // --- Pixel Data ---
    // Now we generate the pixel data, byte by byte.
    // We will create a colorful gradient.
    // The outer loop iterates over each row, from top to bottom (0 to height-1).
    for j in 0..height {
        // The inner loop iterates over each pixel in the row, from left to right.
        for i in 0..width {
            // We create a color based on the pixel's position.
            // The `as u8` cast is important. It converts the result of the calculation
            // into a single byte (0-255).
            // The red component will change with the horizontal position `i`.
            let r = i as u8;
            // The green component will change with the vertical position `j`.
            let g = j as u8;
            // The blue component will be a fixed value for this example.
            let b = 128 as u8;

            // We create a small array holding our three color bytes.
            let pixel = [r, g, b];

            // We write these three bytes directly to standard output.
            // `write_all` ensures that the entire buffer is written.
            stdout.write_all(&pixel)?;
        }
    }

    // `Ok(())` signifies that the program completed successfully.
    Ok(())
}