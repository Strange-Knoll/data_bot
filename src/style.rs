use std::{io::stdout, error::Error};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, 
    event,
};


pub fn styled_print(fg:Color, bg:Color, string:&str)->Result<(), Box<dyn Error>>{
    execute!(
        stdout(),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(string),
        ResetColor
    )?;
    Ok(())
}

pub fn styled_println(fg:Color, bg:Color, string:&str)->Result<(), Box<dyn Error>>{
    styled_print(fg,bg,string)?;
    execute!(stdout(), ResetColor, Print("\n"))?;
    Ok(())
}