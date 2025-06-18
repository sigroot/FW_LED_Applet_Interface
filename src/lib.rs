// Written by Sigroot
// Rust interface structure for Framework LED Matrix

use std::net::{SocketAddr, TcpStream};
use std::io::{Result, Error, ErrorKind};
use serde::{Serialize};

pub struct AppletInterface {
    stream: TcpStream,
    app_num: u8,
    goban: [[u8; 9]; 11],
    separator_type: Separator,
    separator: [u8; 9],
}

impl AppletInterface {
    pub fn new(port: u16, app_num: u8, separator_type: Separator) -> Result<Self> {
        let local_addr = SocketAddr::from(([127,0,0,1], port));
        let stream = TcpStream::connect(local_addr)?;
        if app_num > 2 {
            return Err(Error::new(ErrorKind::InvalidInput, "app_num maximum is 2"))
        }

        let mut command = Command {
            opcode: Opcode::CreateApplet,
            app_num: app_num,
            parameters: match separator_type {
                Separator::Empty => [0].to_vec(),
                Separator::Solid => [1].to_vec(),
                Separator::Dotted => [2].to_vec(),
                Separator::Variable => [3].to_vec(),                
            },
        };

        serde_json::to_writer(&stream, &command)?;

        Ok(Self {
            stream: stream,
            app_num: app_num,
            goban: [[0; 9]; 11],
            separator_type: separator_type,
            separator: [0; 9],
        })
    }

    pub fn set_goban(&mut self, array: [[u8; 9]; 11]) {
        self.goban = array.clone();
    }

    pub fn set_point(&mut self, x: usize, y: usize, value: u8) -> Result<()> {
        let row: &mut [u8; 9] = self.goban.get_mut(y).ok_or(Error::new(ErrorKind::InvalidInput, "Invalid row index"))?;
        let pixel: &mut u8 = row.get_mut(x).ok_or(Error::new(ErrorKind::InvalidInput, "Invalid column index"))?;
        *pixel = value;
        Ok(())
    }

    pub fn get_goban(&self) -> &[[u8; 9]; 11] {
        &self.goban
    }

    pub fn write_goban(&self) -> Result<()> {
        let mut command = Command {
            opcode: Opcode::UpdateGoban,
            app_num: self.app_num,
            parameters: vec![0; 99],
        };

        for i in 0..11 {
            for j in 0..9 {
                command.parameters[i*9+j] = self.goban[i][j];
            }
        }

        Ok(serde_json::to_writer(&self.stream, &command)?)
    }

    pub fn set_bar(&mut self, array: [u8; 9]) {
        self.separator = array.clone();
    }

    pub fn get_bar(&self) -> &[u8; 9] {
        &self.separator
    }

    pub fn write_bar(&self) -> Result<()> {
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

        Ok(serde_json::to_writer(&self.stream, &command)?)
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
    UpdateGoban,
    UpdateBar,
}

#[cfg(test)]
mod tests {
    use crate::AppletInterface;
    use crate::Separator;
    #[test]
    fn test_image() {
        let mut pattern1 = [[0; 9]; 11];
        let mut pattern2 = [[0; 9]; 11];
        let mut pattern3 = [[0; 9]; 11];
        
        for i in 0..11 {
            for j in 0..9 {
                pattern1[i][j] = (i * 11 + j + 1) as u8;
            }
        }

        for i in 0..11 {
            for j in 0..9 {
                pattern2[i][j] = (i * 11 + j + 100) as u8;
            }
        }

        for i in 0..11 {
            for j in 0..9 {
                pattern3[i][j] = std::cmp::min(i * 11 + j + 200, 255) as u8;
            }
        }

        let bar = [255, 150, 50, 10, 0, 10, 50, 150, 255];
        let mut applet1 = AppletInterface::new(27072, 0, Separator::Solid).unwrap();
        applet1.set_goban(pattern1);
        applet1.write_goban().unwrap();

        let mut applet2 = AppletInterface::new(27072, 1, Separator::Dotted).unwrap();
        applet2.set_goban(pattern2);
        applet2.write_goban().unwrap();

        let mut applet3 = AppletInterface::new(27072, 2, Separator::Variable).unwrap();
        applet3.set_goban(pattern3);
        applet3.set_bar(bar);
        applet3.write_goban().unwrap();
        applet3.write_bar().unwrap();
    }
}
