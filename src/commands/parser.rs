use super::commands::Command;
use super::commands::CommandType;

enum ArgResult {
    Error,
    EOF,
}
pub struct Parser {}
impl Parser {
    pub fn parse<'a>(command_data: &'a [u8]) -> Result<Command<'a>, ()> {
        if command_data.len() < 2 {
            return Err(());
        }
        let mut args_part = command_data.split_at(2).1;
        let mut args: Vec<&[u8]> = Vec::new();
        let loop_result = loop {
            let result = Self::get_arg(args_part);
            match result {
                Err(ArgResult::EOF) => break Ok(()),
                Err(ArgResult::Error) => break Err(()),
                Ok((arg, remainder)) => {
                    args_part = remainder;
                    args.push(arg);
                    continue;
                }
            };
        };
        match loop_result {
            Ok(()) => Ok(Command {
                command_type: CommandType::from([command_data[0], command_data[1]]),
                arguments: args,
            }),
            Err(()) => Err(()),
        }
    }
    fn get_arg(mut data: &[u8]) -> Result<(&[u8], &[u8]), ArgResult> {
        if data.len() == 0 {
            return Err(ArgResult::EOF);
        }
        if data.len() < 4 {
            println!("Data too small, not even length");
            return Err(ArgResult::Error);
        }
        let split = data.split_at(4);
        data = split.1;
        let len_split: [u8; 4] = [split.0[0], split.0[1], split.0[2], split.0[3]];
        let len = u32::from_le_bytes(len_split);
        //println!("Expected arg len {:?}", len);
        match data.len() {
            l if l < len as usize => Err(ArgResult::Error),
            _ => Ok(data.split_at(len as usize)),
        }
    }
}
