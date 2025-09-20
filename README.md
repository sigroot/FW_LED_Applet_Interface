# Framework LED Matrix Applet Interface
![Insert Gif Here](video/matrix.gif)
## Preamble
**THIS IS UNOFFICIAL SOFTWARE. I AM NOT AFFILIATED WITH FRAMEWORK**

This repository contains a rust library for communicating with the [FW_LED_Matrix_Board](https://github.com/sigroot/FW_LED_Matrix_Board) server. A single 'AppletInterface' struct represents either a 9x1 status bar or one of four 9x11 applets on the matrix interface.

This repository requires the [FW_LED_Matrix_Board](https://github.com/sigroot/FW_LED_Matrix_Board) repository. The matrix board server must be running

Thanks to [Ecca](https://community.frame.work/t/use-cases-for-the-led-matrix-module/39171/75) on the Framework Forums for the idea of separating the LED matrix into separate modules.
## Capabilities
This repository contains an 'AppletInterface' struct which can write to one applet of the [FW_LED_Matrix_Board](https://github.com/sigroot/FW_LED_Matrix_Board). When an AppletInterface is created, it is assigned the u8 app_num from 0-3. '0' creates the AppletInterface as the matrix interface's status bar and the struct will only accept modifications to its 9x1 separator bar. 1-3 creates the AppletInterface as one of the matrix interface's grids and the struct will accept modifications to its 9x1 separator bar or its 9x10 grid. 

The AppletInterface struct contains buffers for both its separator bar and its grid. These buffers have setter and getter methods. The struct has methods to write these buffers to the matrix interface. After writing, the AppletInterface will return a u8 error code.

The server can update each applet at roughly 80 frames per second.
### Use Example
```
    use sigroot_applet_interface::*;
    
    fn main() {
        let mut pattern = [[0; 9]; 10];
        
        for i in 0..10 {
            for j in 0..9 {
                pattern[i][j] = (i * 10 + j + 1) as u8;
            }
        }
        
        let status = [255, 175, 125, 100, 75, 50, 25, 12, 0];
        
        let mut applet = AppletInterface::new(27072, 1, Separator::Variable).unwrap();
        
        applet.set_bar(status);
        applet.set_grid(pattern);
        applet.write_bar().unwrap();
        applet.write_grid().unwrap();
    }
```
### Associated Software
[FW_LED_Matrix_Firmware](https://github.com/sigroot/FW_LED_Matrix_Firmware) is Arduino-based firmware and is a prerequisite installation for this library.

[FW_LED_Matrix_Interface](https://github.com/sigroot/FW_LED_Matrix_Interface) is a Rust library for interfacing between this firmware and other Rust programs.

[FW_LED_Matrix_Board](https://github.com/sigroot/FW_LED_Matrix_Board) divides the 9x34 LED Matrix into 3 smaller 9x11 "applets" and provides a language agnostic interface between these applets and other programs.
### Communication
Communication is over TCP

Commands are received with JSON encoded 'Command' structres in the format:
{
    "opcode": "<Command Name>",
    "app_num": <Applet Number (0-2)>,
    "parameters": [x<,y<,...z> (where each value is a u8)]
}

**Commands**:

CreateApplet - Creates a new applet assigned to the requesting TCP stream

Parameters: 1 u8 from 0-3
    0 - Applet separator is empty (all LED's off)

    1 - Applet separator is solid (all LED's on)

    2 - Applet separator is dotted (alternating LED's on & off)

    3 - Applet seprator is variable (default off)

UpdateGrid - Rewrites the current 9x10 applet grid with new values

Parameters: 
    90 u8 representing grid brightnesses - rows then columns (1st 10 is row1, 2nd 10 is row2, etc.)

UpdateBar - Rewrites the current 9x1 applet separator

Parameters:
    9 u8 representing separator brightnesses

    Note: Error 32 returned if bar is not variable

sig_rp2040_board will respond with a single u8 error code (not JSON):

0:	    Command successfully processed

10:	    Failed to read data from stream

20:	    Failed to parse stream data as UTF-8

21:	    Failed to parse stream data as JSON

30:	    Command uses invalid applet number (greater than 2)

31:	    Command attempts to modify applet stream did not create

32:     Attempt to update applet 0 grid

33:	    Error in commanding applet

34:	    Attempt to create new applet when applet already exists

40:	    Invalid separator value when creating applet

255:	Unknown error

### Methods
> new(port: u16, app\_num: u8, separator\_type: Separator) -> Result<Self>

Returns a Result containing a new AppletInterface struct. port is the TCP port over which the struct will communicate with the matrix interface (matrix interface default is 27072). app\_num specifies which applet of the matrix interface this struct will communicate with. separator\_type receives the included Separator enum to determine the whether this struct's separator is Empty, Solid, Dotted, or Variable (the separator must be variable to be updated).

> set_grid(&mut self, array: [[u8; 9]; 10])

Sets this AppletInterface struct's grid buffer with an inputted matrix. array is the inputted matrix of u8 pwm values to which this struct's grid buffer will be set.

> set_point(&mut self, x: usize, y: usize, value: u8) -> Result<()>

Sets this AppletInterface struct's grid buffer with an inputted x and y coordinate pair. x is the LED position from the left. y is the LED position from the top. value is the new pwm value for the LED.

> get_grid(&self) -> &[[u8; 9]; 10]

Returns a reference to this AppletInterface struct's grid buffer.

> write_grid(&mut self) -> Result<()>

Sends a command to the matrix interface to update the associated displayed applet grid with this AppletInterface struct's grid buffer. Returns an error code. 

> set_bar(&mut self, array: [u8; 9])

Sets this AppletInterface struct's separator bar buffer with an inputted matrix. **This will not complete if this struct's separator_type is not Variable**. array is an array of u8 pwm values to which this struct's separator bar buffer will be set.

> get_bar(&self) -> &[u8; 9]

Returns a reference to this AppletInterface struct's separator bar buffer.

> write_bar(&mut self) -> Result<()>

Sends a command to the matrix interface to update the associated displayed applet separator bar with this AppletInterface struct's separator bar buffer. **This will not complete if this struct's separator_type is not Variable**. Returns an error code. 
