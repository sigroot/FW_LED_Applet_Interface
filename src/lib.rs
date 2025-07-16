//! Written by Sigroot
//! sigroot_applet_interface - A Rust interface structure for Framework LED Matrix
//! 
//! Interface library contains an AppletInterface struct for communicating with
//! the LED matrix board program
//! 
//! new() requires the localhost's port that is currently used by the LED matrix
//! board program, the applet number being interfaced with (0 is the top row
//! status bar, 1-3 are the applets, 4-256 are invalid), and the separator type
//! (if not variable, attempts to use set_bar() will error)
//!
//! set_grid(), set_point(), and set_bar() modify the struct's internal grid and
//! separator while write_grid() and write_bar() send both to the LED matrix board
//! program respectively
//! 
//! Actual communication is in the following format:
//!
//! Communication is over TCP
//!
//! Commands are sent with JSON encoded 'Command' strucutres in the format:
//! ```ignore
//! {
//!     "opcode": "<Command Name>",
//!     "app_num": <Applet Number (0-2)>,
//!     "parameters": [x<,y<,...z> (where each value is a u8)]
//! }
//! ```
//!
//! Commands:
//! CreateApplet - Creates a new applet assigned to the requesting TCP stream
//!     Parameters: 1 u8 from 0-3
//!         0 - Applet separator is empty (all LED's off)
//!         1 - Applet separator is solid (all LED's on)
//!         2 - Applet separator is dotted (alternating LED's on & off)
//!         3 - Applet seprator is variable (default off)
//! 
//! UpdateGrid - Rewrites the current 9x10 applet grid with new values
//!     Parameters: 90 u8 representing grid brightnesses - rows then columns 
//!                 (1st 10 is row1, 2nd 10 is row2, etc.)
//! 
//! UpdateBar - Rewrites the current 9x1 applet separator
//!     Parameters: 9 u8 representing separator brightnesses
//!     Note: Error 32 returned if bar is not variable
//!
//! sig_rp2040_board will respond with a single u8 error code (not JSON):
//! 0:	    Command successfully processed
//! 10:	    Failed to read data from stream
//! 20:	    Failed to parse stream data as UTF-8
//! 21:	    Failed to parse stream data as JSON
//! 30:	    Command uses invalid applet number (greater than 2)
//! 31:	    Command attempts to modify applet stream did not create
//! 32:     Attempt to update applet 0 grid
//! 33:	    Error in commanding applet
//! 34:	    Attempt to create new applet when applet already exists
//! 40:	    Invalid separator value when creating applet
//! 255:	Unknown error

use std::net::{SocketAddr, TcpStream};
use std::io::{Result, Error, ErrorKind, Read, Write};
use serde::{Serialize};

pub struct AppletInterface {
    stream: TcpStream,
    app_num: u8,
    grid: [[u8; 9]; 10],
    separator_type: Separator,
    separator: [u8; 9],
}

impl AppletInterface {
    pub fn new(port: u16, app_num: u8, separator_type: Separator) -> Result<Self> {
        // Fail if app_num is invalid
        if app_num > 3 {
            return Err(Error::new(ErrorKind::InvalidInput, "app_num maximum is 3"))
        }

        // Create TCP stream
        let local_addr = SocketAddr::from(([127,0,0,1], port));
        let mut stream = TcpStream::connect(local_addr)?;

        // Generate CreateApplet command
        let command = Command {
            opcode: Opcode::CreateApplet,
            app_num: app_num,
            parameters: match separator_type {
                Separator::Empty => [0].to_vec(),
                Separator::Solid => [1].to_vec(),
                Separator::Dotted => [2].to_vec(),
                Separator::Variable => [3].to_vec(),                
            },
        };

        // Send command over stream as json (faster if not using serde_json::to_writer())
        let json_string = serde_json::to_string(&command)?;
        stream.write_all(json_string.as_bytes());

        let mut buffer: [u8; 1] = [255];
        stream.read_exact(&mut buffer)?;
        if buffer[0] != 0 {return Err(Error::new(ErrorKind::ConnectionRefused, "Board refused connection"))}

        // Return applet
        Ok(Self {
            stream: stream,
            app_num: app_num,
            grid: [[0; 9]; 10],
            separator_type: separator_type,
            separator: [0; 9],
        })
    }

    pub fn set_grid(&mut self, array: [[u8; 9]; 10]) {
        self.grid = array.clone();
    }

    pub fn set_point(&mut self, x: usize, y: usize, value: u8) -> Result<()> {
        let row: &mut [u8; 9] = self.grid.get_mut(y).ok_or(Error::new(ErrorKind::InvalidInput, "Invalid row index"))?;
        let pixel: &mut u8 = row.get_mut(x).ok_or(Error::new(ErrorKind::InvalidInput, "Invalid column index"))?;
        *pixel = value;
        Ok(())
    }

    pub fn get_grid(&self) -> &[[u8; 9]; 10] {
        &self.grid
    }

    pub fn write_grid(&mut self) -> Result<()> {
        // Create command to write a new grid
        let mut command = Command {
            opcode: Opcode::UpdateGrid,
            app_num: self.app_num,
            parameters: vec![0; 90],
        };


        for i in 0..10 {
            for j in 0..9 {
                command.parameters[i*9+j] = self.grid[i][j];
            }
        }

        // Send command over stream as json (faster if not using serde_json::to_writer())
        let json_string = serde_json::to_string(&command)?;
        self.stream.write_all(json_string.as_bytes());

        let mut buffer = [255; 1];
        self.stream.read_exact(&mut buffer)?;
        
        match buffer[0] {
            0 => Ok(()),
            10 => Err(Error::new(ErrorKind::InvalidData, "Board could not read data from stream")),
            20 => Err(Error::new(ErrorKind::InvalidData, "Data not in UTF-8")),
            21 => Err(Error::new(ErrorKind::InvalidData, "Data not in JSON")),
            30 => Err(Error::new(ErrorKind::InvalidInput, "Command uses invalid applet number")),
            31 => Err(Error::new(ErrorKind::InvalidInput, "Command uses wrong applet number")),
            32 => Err(Error::new(ErrorKind::Other, "Command failed")),
            33 => Err(Error::new(ErrorKind::AlreadyExists, "Applet already exists during create applet command")),
            40 => Err(Error::new(ErrorKind::InvalidInput, "Command uses invalid separator number")),
            _ => Err(Error::new(ErrorKind::Other, "Unkown Error")),
        }
    }

    pub fn set_bar(&mut self, array: [u8; 9]) {
        self.separator = array.clone();
    }

    pub fn get_bar(&self) -> &[u8; 9] {
        &self.separator
    }

    pub fn write_bar(&mut self) -> Result<()> {
        match self.separator_type {
            Separator::Variable => (),
            _ => return Err(Error::new(ErrorKind::InvalidInput, "separator not variable")),
        }

        let mut command = Command {
            opcode: Opcode::UpdateBar,
            app_num: self.app_num,
            parameters: vec![0; 9],
        };

        for i in 0..9 {
            command.parameters[i] = self.separator[i];
        }

        // Send command over stream as json (faster if not using serde_json::to_writer())
        let json_string = serde_json::to_string(&command)?;
        self.stream.write_all(json_string.as_bytes());

        let mut buffer = [255; 1];
        self.stream.read_exact(&mut buffer)?;
        match buffer[0] {
            0 => Ok(()),
            10 => Err(Error::new(ErrorKind::InvalidData, "Board could not read data from stream")),
            20 => Err(Error::new(ErrorKind::InvalidData, "Data not in UTF-8")),
            21 => Err(Error::new(ErrorKind::InvalidData, "Data not in JSON")),
            30 => Err(Error::new(ErrorKind::InvalidInput, "Command uses invalid applet number")),
            31 => Err(Error::new(ErrorKind::InvalidInput, "Command uses wrong applet number")),
            32 => Err(Error::new(ErrorKind::Other, "Command failed")),
            33 => Err(Error::new(ErrorKind::AlreadyExists, "Applet already exists during create applet command")),
            40 => Err(Error::new(ErrorKind::InvalidInput, "Command uses invalid separator number")),
            _ => Err(Error::new(ErrorKind::Other, "Unkown Error")),
        }
    }
}

pub enum Separator {
    Empty,
    Solid,
    Dotted,
    Variable,
}

#[derive(Serialize)]
pub struct Command {
    pub opcode: Opcode,
    pub app_num: u8,
    pub parameters: Vec<u8>,
}

#[derive(Serialize, PartialEq, Eq)]
pub enum Opcode {
    CreateApplet,
    UpdateGrid,
    UpdateBar,
}

#[cfg(test)]
mod tests {
    use crate::AppletInterface;
    use crate::Separator;

    const FPS: u128 = 60;

    #[test]
    fn test_image() {
        let mut pattern1 = [[0; 9]; 10];
        let mut pattern2 = [[0; 9]; 10];
        let mut pattern3 = [[0; 9]; 10];
        
        for i in 0..10 {
            for j in 0..9 {
                pattern1[i][j] = (i * 10 + j + 1) as u8;
            }
        }

        for i in 0..10 {
            for j in 0..9 {
                pattern2[i][j] = (i * 10 + j + 100) as u8;
            }
        }

        for i in 0..10 {
            for j in 0..9 {
                pattern3[i][j] = std::cmp::min(i * 11 + j + 200, 255) as u8;
            }
        }

        let status = [255, 175, 125, 100, 75, 50, 25, 12, 0];
        let mut applet0 = AppletInterface::new(27072, 0, Separator::Variable).unwrap();
        applet0.set_bar(status);
        applet0.write_bar().unwrap();

        let mut applet1 = AppletInterface::new(27072, 1, Separator::Solid).unwrap();
        applet1.set_grid(pattern1);
        applet1.write_grid().unwrap();

        let mut applet2 = AppletInterface::new(27072, 2, Separator::Dotted).unwrap();
        applet2.set_grid(pattern2);
        applet2.write_grid().unwrap();

        let bar = [255, 150, 50, 10, 0, 10, 50, 150, 255];
        let mut applet3 = AppletInterface::new(27072, 3, Separator::Variable).unwrap();
        applet3.set_grid(pattern3);
        applet3.set_bar(bar);
        applet3.write_grid().unwrap();
        applet3.write_bar().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(3000));
    }
    #[test]
    fn test_grid_animation() {
        let mut applet = AppletInterface::new(27072, 1, Separator::Variable).unwrap();

        for i in 0..90 {
            let start = std::time::SystemTime::now();
            
            let mut grid = [[0;9];10];
            grid[i/9][i%9] = 255;
            applet.set_grid(grid);
            applet.write_grid().unwrap();

            while std::time::Duration::as_micros(&std::time::SystemTime::now().duration_since(start).unwrap()) < 1000000/FPS {}
            println!("Grid: {:.2}", 1000000.0/(std::time::Duration::as_micros(&std::time::SystemTime::now().duration_since(start).unwrap())as f64));
        }
    }

    #[test]
    fn test_bar_animation() {
        let mut applet = AppletInterface::new(27072, 2, Separator::Variable).unwrap();

        for i in 1..100 {
            let start = std::time::SystemTime::now();

            applet.set_bar([(5*(i as u32) +1%255) as u8, (5*(i as u32) +25%255) as u8, (5*(i as u32) +50%255) as u8, (5*(i as u32) +75%255) as u8, (5*(i as u32) +100%255) as u8, (5*(i as u32) +125%255) as u8, (5*(i as u32) +150%255) as u8, (5*(i as u32) +175%255) as u8, (5*(i as u32) +200%255) as u8]);
            applet.write_bar().unwrap();

            while std::time::Duration::as_micros(&std::time::SystemTime::now().duration_since(start).unwrap()) < 1000000/FPS {}
            println!("Bar:  {:.2}", 1000000.0/(std::time::Duration::as_micros(&std::time::SystemTime::now().duration_since(start).unwrap())as f64));
        }  
    }

    #[test]
    fn applet_0_grid_fail() {
        let mut applet = AppletInterface::new(27072, 0, Separator::Variable).unwrap();

        applet.set_bar([25; 9]);
        assert!(applet.write_bar().is_ok());
        applet.set_grid([[5; 9]; 10]);
        assert!(applet.write_grid().is_err());
    }
}
