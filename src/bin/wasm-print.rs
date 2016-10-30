extern crate wasm;

use std::fs::File;
use std::io::Read;
use wasm::reader::*;
use std::env::args;

fn main() {
    let mut buf = vec![];
    let name = match args().skip(1).next() {
        None => {
            println!("usage: wasm-print <input binary>");
            return
        },
        Some(x) => x
    };
    File::open(&name)
        .expect(&format!("Failed to open {}", name))
        .read_to_end(&mut buf)
        .expect(&format!("Failed to read {}", name));
    let module = match Module::new(&buf) {
        Err(e) => {
            println!("{}", e);
            return
        },
        Ok(v) => v
    };
    for section in module.sections() {
        let section = match section {
            Err(e) => {
                println!("{}", e);
                break
            },
            Ok(v) => v
        };
        println!("--- section {:?} {} ----------", section.id, section.name);
        let content = match section.content() {
            Ok(content) => content,
            Err(e) => {
                println!("{}", e);
                break
            }
        };
        match content {
            SectionContent::Type(types) => {
                for ty in types.entries() {
                    let ty = match ty {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    let func = match ty {
                        TypeEntry::Function(func) => func,
                    };
                    print!("Function returning {:?}, accepting:", func.return_type);
                    for param in func.params() {
                        let param = match param {
                            Err(e) => {
                                println!("{}", e);
                                break
                            },
                            Ok(v) => v
                        };
                        print!(" {:?}", param);
                    }
                    println!("");
                }
            },
            SectionContent::Import(imports) => {
                for entry in imports.entries() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    print!("{}::{}: ", entry.module, entry.field);
                    match entry.contents {
                        ImportEntryContents::Function(func) => println!("function type={}", func),
                        ImportEntryContents::Table { element_type, ref limits } => {
                            println!(
                                "table type={}, initial={}, max={:?}",
                                element_type, limits.initial, limits.maximum
                            );
                        },
                        ImportEntryContents::Memory(ref limits) => {
                            println!(
                                "memory initial={}, max={:?}",
                                limits.initial, limits.maximum
                            );
                        },
                        ImportEntryContents::Global { ty, mutable } => {
                            println!("global ty={:?}, mutable={}", ty, mutable);
                        }
                    }
                }
            },
            SectionContent::Function(functions) => {
                for (i, entry) in functions.types().enumerate() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    println!("function {}: type={}", i, entry);
                }
            },
            SectionContent::Global(globals) => {
                let mut i = 0;
                for entry in globals.entries() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    match entry {
                        GlobalEntryEither::Entry(entry) => {
                            println!(
                                "global {}: ty={:?}, mutable={}",
                                i, entry.ty, entry.mutable
                            );
                            i += 1;
                        },
                        GlobalEntryEither::Op(op) => {
                            println!("  {:?}", op);
                        }
                    }
                }
            },
            SectionContent::Export(exports) => {
                for entry in exports.entries() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    println!(
                        "{}: kind={:?}, index={}",
                        entry.field, entry.kind, entry.index
                    );
                }
            },
            SectionContent::Elements(elements) => {
                #[derive(PartialEq, Eq)]
                enum State {
                    Default,
                    Elem
                }
                let mut state = State::Default;
                for entry in elements.entries() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    match entry {
                        ElementEntry::Index(index) => {
                            if state != State::Default {
                                println!("");
                                state = State::Default;
                            }
                            println!("element index={}", index);
                        },
                        ElementEntry::Op(op) => {
                            println!("  {:?}", op);
                        },
                        ElementEntry::Elem(elem) => {
                            if state != State::Elem {
                                print!(" ");
                                state = State::Elem;
                            }
                            print!(" {}", elem);
                        }
                    }
                }
                if state != State::Default {
                    println!("");
                }
            },
            SectionContent::Code(code) => {
                for (i, entry) in code.entries().enumerate() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    println!("function {}", i);
                    for part in entry.contents() {
                        let part = match part {
                            Err(e) => {
                                println!("{}", e);
                                break
                            },
                            Ok(v) => v
                        };
                        match part {
                            FunctionPart::Local(local) => {
                                println!(
                                    "  local {:?} x {}",
                                    local.ty, local.count
                                );
                            },
                            FunctionPart::Op(op) => {
                                println!("  {:?}", op);
                            }
                        }
                    }
                }
            },
            SectionContent::Data(data) => {
                for entry in data.entries() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    match entry {
                        DataEntry::Index(index) => {
                            println!("data for heap {}", index);
                            println!("  offset:");
                        },
                        DataEntry::Op(op) => {
                            println!("    {:?}", op);
                        },
                        DataEntry::Data(data) => {
                            println!("  value:");
                            for line in data.chunks(16).map(|chunk| {
                                chunk.iter().map(
                                    |x| format!("{:02x}", x)
                                ).collect::<Vec<_>>().join(" ")
                            }) {
                                println!("    {}", line);
                            }
                        },
                    }
                }
            },
            SectionContent::Name(names) => {
                for entry in names.entries() {
                    let entry = match entry {
                        Err(e) => {
                            println!("{}", e);
                            break
                        },
                        Ok(v) => v
                    };
                    match entry {
                        NameEntry::Function(name) => {
                            println!("{}", name);
                        },
                        NameEntry::Local(name) => {
                            println!("  {}", name);
                        }
                    }
                }
            },
            _ => (),
        }
    }
}
