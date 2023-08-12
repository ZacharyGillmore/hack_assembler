use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect(); // arg[1] is the input file name/path 
    let mut instructions: Vec<String> = Vec::new();
    let mut symbols: HashMap<String, u16> = HashMap::from(
        // numbers are memory locations that the symbols refer to by default
        [(String::from("SCREEN"),16384), 
        (String::from("KBD"),24576),
        (String::from("SP"), 0),
        (String::from("LCL"), 1),
        (String::from("ARG"), 2),
        (String::from("THIS"), 3),
        (String::from("THAT"), 4)]
    );
    for i in 0..16 {
        // add R0-R15 to symbols
        symbols.insert(format!("R{}",i), i);
    }
    // put file reading in block to drop all comments from memory
    {
        let filename = args[1].as_str(); //input file
        println!("Reading file {}", filename);

        let file_contents = fs::read_to_string(filename).expect("Couldn't read file");
        let file_contents = file_contents.lines();
        for instr in file_contents {
            let instr = {
                let instr = String::from(instr);
                let instr = {
                    match instr.find("//") {
                        Some(i) => &instr[..i],
                        None => instr.as_str(),
                    }
                 };
                String::from(instr.trim())
            };
            let first_char = instr.chars().next();
            match first_char {
                Some('/') => (),
                Some('(') => {
                    let symb_name = String::from(&instr[1..instr.len() - 1]);
                    symbols.insert(symb_name, instructions.len() as u16);
                }
                Some(_) => instructions.push(instr),
                None => (),
            }
        }

        let mut var_addr: u16 = 16;
        for instr in &instructions {
            // loop agian to create variables
            let first_char = instr.chars().next();
            if let Some('@') = first_char {
                // check if variable and add to symbols
                let var_name = String::from(&instr[1..]);
                let is_symbol = !(var_name.chars().all(|x| x.is_numeric()));
                if is_symbol && !symbols.contains_key(&var_name) {
                    symbols.insert(var_name, var_addr);
                    var_addr += 1;
                }
            }
        }
    }
    let mut out_f = {
        let out_f_name = format!("{}.hack", (&args[1][..args[1].find('.').unwrap()]));
        println!("creating and writing to file: {}", out_f_name);
        fs::File::create(out_f_name).unwrap()
    };
    for instr in instructions {
        let first_char = instr.chars().next().unwrap();
        if first_char == '@' {
            out_f
                .write_all(
                    format!("{}\n", a_to_binary(String::from(&instr[1..]), &symbols)).as_bytes(),
                )
                .unwrap();
        } else {
            out_f
                .write_all(format!("{}\n", c_to_binary(instr)).as_bytes())
                .unwrap();
        }
    }
}

fn a_to_binary(instr: String, symbols: &HashMap<String, u16>) -> String {
    /* converts an @ instruction to binary
    assumes @ character is removed */
    let mut n_bin = {
        if symbols.contains_key(&instr) {
            let num: u16 = *symbols.get(&instr).unwrap();
            format!("{num:b}")
        } else {
            let num: u16 = instr.parse().unwrap();
            format!("{num:b}")
        }
    };
    while n_bin.len() < 16 {
        n_bin.insert(0, '0');
    }
    n_bin
}

fn c_to_binary(instr: String) -> String {
    /* converts dest=comp;jmp instuction to binary*/
    let mut out = String::from("111");
    let fields = {
        let dest = instr.find('=').map(|i| &instr[..i]);
        let cond = {
            let i1 = match instr.find('=') {
                Some(i1) => i1 + 1,
                None => 0,
            };
            let out = match instr.find(';') {
                Some(i2) => &instr[i1..i2],
                None => &instr[i1..],
            };
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        };
        let jump = instr.split(';').nth(1);
        vec![cond, dest, jump]
    };
    out.push_str(match fields[0].unwrap_or("null") {
        "0" => "0101010",
        "1" => "0111111",
        "-1" => "0111010",
        "D" => "0001100",
        "A" => "0110000",
        "M" => "1110000",
        "!D" => "0001101",
        "!A" => "0110001",
        "!M" => "1110001",
        "-D" => "0001111",
        "-A" => "0110011",
        "-M" => "1110011",
        "D+1" => "0011111",
        "A+1" => "0110111",
        "M+1" => "1110111",
        "D-1" => "0001110",
        "A-1" => "0110010",
        "M-1" => "1110010",
        "D+A" => "0000010",
        "D+M" => "1000010",
        "D-A" => "0010011",
        "D-M" => "1010011",
        "A-D" => "0000111",
        "M-D" => "1000111",
        "D&A" => "0000000",
        "D&M" => "1000000",
        "D|A" => "0010101",
        "D|M" => "1010101",
        _ => panic!("invalid instruction!"),
    });
    out.push_str(match fields[1].unwrap_or("nul") {
        "nul" => "000",
        "M" => "001",
        "D" => "010",
        "MD" => "011",
        "A" => "100",
        "AM" => "101",
        "AD" => "110",
        "AMD" => "111",
        _ => panic!("invalid instruction!"),
    });
    out.push_str(match fields[2].unwrap_or("nul") {
        "nul" => "000",
        "JGT" => "001",
        "JEQ" => "010",
        "JGE" => "011",
        "JLT" => "100",
        "JNE" => "101",
        "JLE" => "110",
        "JMP" => "111",
        _ => panic!("invalid instruction!"),
    });
    out
}
