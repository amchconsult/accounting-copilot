use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{fs::{OpenOptions, File}, io::{self, BufRead, BufReader, Write}, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JournalEntry {
    id: u32,
    journal_date: NaiveDate,
    account_id: u32,
    amount_debt: f64,
    amount_credit: f64,
    total: f64,
    reconciled: bool,
    isdeleted: String,
}

struct AccountingSystem {
    entries: Vec<JournalEntry>,
    next_id: u32,
    filename: String,
}

impl AccountingSystem {
    fn new(filename: &str) -> Self {
        let mut sys = Self {
            entries: Vec::new(),
            next_id: 1,
            filename: filename.to_string(),
        };
        sys.load();
        sys
    }

    fn load(&mut self) {
        self.entries.clear();
        if Path::new(&self.filename).exists() {
            let file = File::open(&self.filename).expect("Cannot open entries file");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(l) = line {
                    if let Ok(entry) = serde_json::from_str::<JournalEntry>(&l) {
                        if entry.id >= self.next_id {
                            self.next_id = entry.id + 1;
                        }
                        self.entries.push(entry);
                    }
                }
            }
        }
    }

    fn save(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.filename)
            .expect("Cannot open for writing");
        for entry in &self.entries {
            let json = serde_json::to_string(entry).unwrap();
            writeln!(file, "{}", json).unwrap();
        }
    }

    fn add_entry(&mut self, mut entry: JournalEntry) {
        entry.id = self.next_id;
        self.next_id += 1;
        // Calculate total
        entry.total = entry.amount_debt - entry.amount_credit;
        self.entries.push(entry);
        self.save();
    }

    fn list_entries(&self) -> Vec<&JournalEntry> {
        self.entries.iter().filter(|e| e.isdeleted == "no").collect()
    }

    fn update_entry(&mut self, id: u32, mut updated: JournalEntry) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id && e.isdeleted == "no") {
            // Recalculate total
            updated.total = updated.amount_debt - updated.amount_credit;
            *entry = updated;
            entry.id = id;
            self.save();
            true
        } else {
            false
        }
    }

    fn delete_entry(&mut self, id: u32) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id && e.isdeleted == "no") {
            entry.isdeleted = "yes".to_string();
            self.save();
            true
        } else {
            false
        }
    }

    fn get_entry(&self, id: u32) -> Option<&JournalEntry> {
        self.entries.iter().find(|e| e.id == id && e.isdeleted == "no")
    }
}

fn prompt(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn print_commands() {
    println!("\nCommands: add, list, update, delete, get, exit");
}

fn main() {
    let mut system = AccountingSystem::new("entries.txt");

    println!("Welcome to Accounting Copilot CLI!");
    print_commands();

    loop {
        let cmd = prompt("\n> ").to_lowercase();
        match cmd.as_str() {
            "add" => {
                let date_str = prompt("journal_date (YYYY-MM-DD): ");
                let journal_date = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => {
                        println!("Invalid date format.");
                        print_commands();
                        continue;
                    }
                };
                let account_id: u32 = prompt("account_id: ").parse().unwrap_or(0);
                let amount_debt: f64 = prompt("amount_debt: ").parse().unwrap_or(0.0);
                let amount_credit: f64 = prompt("amount_credit: ").parse().unwrap_or(0.0);
                let reconciled = prompt("reconciled (true/false): ") == "true";
                let entry = JournalEntry {
                    id: 0,
                    journal_date,
                    account_id,
                    amount_debt,
                    amount_credit,
                    total: amount_debt - amount_credit, // for clarity, but add_entry also ensures this
                    reconciled,
                    isdeleted: "no".to_string(),
                };
                system.add_entry(entry);
                println!("Entry added.");
                print_commands();
            }
            "list" => {
                println!("Current Entries:");
                for entry in system.list_entries() {
                    println!("{:?}", entry);
                }
                print_commands();
            }
            "get" => {
                let id: u32 = prompt("id: ").parse().unwrap_or(0);
                if let Some(entry) = system.get_entry(id) {
                    println!("{:?}", entry);
                } else {
                    println!("Entry not found.");
                }
                print_commands();
            }
            "update" => {
                let id: u32 = prompt("id: ").parse().unwrap_or(0);
                if let Some(orig) = system.get_entry(id).cloned() {
                    let date_str = prompt(&format!("journal_date (YYYY-MM-DD) [{}]: ", orig.journal_date));
                    let journal_date = if date_str.is_empty() {
                        orig.journal_date
                    } else {
                        match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                            Ok(d) => d,
                            Err(_) => {
                                println!("Invalid date format.");
                                print_commands();
                                continue;
                            }
                        }
                    };
                    let account_id: u32 = prompt(&format!("account_id [{}]: ", orig.account_id)).parse().unwrap_or(orig.account_id);
                    let amount_debt: f64 = prompt(&format!("amount_debt [{}]: ", orig.amount_debt)).parse().unwrap_or(orig.amount_debt);
                    let amount_credit: f64 = prompt(&format!("amount_credit [{}]: ", orig.amount_credit)).parse().unwrap_or(orig.amount_credit);
                    let reconciled = prompt(&format!("reconciled (true/false) [{}]: ", orig.reconciled)).parse().unwrap_or(orig.reconciled);
                    let updated = JournalEntry {
                        id,
                        journal_date,
                        account_id,
                        amount_debt,
                        amount_credit,
                        total: amount_debt - amount_credit, // for clarity, but update_entry also ensures this
                        reconciled,
                        isdeleted: "no".to_string(),
                    };
                    if system.update_entry(id, updated) {
                        println!("Entry updated.");
                    } else {
                        println!("Update failed.");
                    }
                } else {
                    println!("Entry not found.");
                }
                print_commands();
            }
            "delete" => {
                let id: u32 = prompt("id: ").parse().unwrap_or(0);
                if system.delete_entry(id) {
                    println!("Entry deleted.");
                } else {
                    println!("Delete failed.");
                }
                print_commands();
            }
            "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command.");
                print_commands();
            }
        }
    }
}