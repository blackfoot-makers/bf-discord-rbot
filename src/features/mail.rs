//! Discontinued
//! Connect to a mail account, and track incoming mails.
//! Allow discord user to check, assign and resolve received mails.

use core::connection::CHANNEL_MAILS;
use imap::client::Client;
use imap::Fetch;
use native_tls::TlsConnector;
use std::str;
use std::{thread, time};
use time::{now, strptime, Duration};
use {CREDENTIALS_FILE, MAIL_INFO_FILE, MAIL_LOCK};

/// Stored info on the session and states of the mails.
#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    mail_number: u32,
    pub resolved: Vec<(u32, (String, String, String), String)>,
    pub assigned: Vec<(u32, (String, String, String), String)>,
    pub unassigned: Vec<(u32, (String, String, String))>,
}

impl Info {
    pub fn display_assigned(&self, filter: String) -> String {
        let mut result = String::from("Assigned mails:\n");
        for (id, (from, subject, date), who) in &self.assigned {
            if filter == "all" || filter == *who {
                let time = strptime(date, "%a, %d %b %Y %T").unwrap();
                let monthfromnow = now() - Duration::days(30);
                if time > monthfromnow {
                    result.push_str(&format!(
                        "Id: {} | Subject: **{}** | From: {} | Assigned to {} [ {} ]\n",
                        id,
                        subject,
                        from,
                        who,
                        time.strftime("%d %b").unwrap()
                    ));
                }
            }
        }
        result
    }

    pub fn display_resolved(&self, filter: String) -> String {
        let mut result = String::from("Resolved mails:\n");
        for (_id, (from, subject, date), who) in &self.resolved {
            if filter == "all" || filter == *who {
                let time = strptime(date, "%a, %d %b %Y %T").unwrap();
                result.push_str(&format!(
                    "Subject: **{}** | From: {} | Assigned to {} [ {} ]\n",
                    subject,
                    from,
                    who,
                    time.strftime("%d %b").unwrap()
                ));
            }
        }
        result
    }

    pub fn display_unassigned(&self) -> String {
        if self.unassigned.len() == 0 {
            return "No unassigned mails.".to_string();
        }
        let mut result = String::from("Unassigned mails:\n");
        for (id, (from, subject, date)) in &self.unassigned {
            let time = strptime(date, "%a, %d %b %Y %T").unwrap();
            let monthfromnow = now() - Duration::days(30);
            if time > monthfromnow {
                result.push_str(&format!(
                    "Id: {} | Subject: **{}** | From: {} [ {} ]\n",
                    id,
                    subject,
                    from,
                    time.strftime("%d %b").unwrap()
                ));
            }
        }
        result
    }

    pub fn new() -> Info {
        Info {
            mail_number: 0,
            resolved: Vec::new(),
            assigned: Vec::new(),
            unassigned: Vec::new(),
        }
    }
}

/// Parse messages from emails and fill apropriate vars
fn parse_rfc(date: &mut String, from: &mut String, subject: &mut String, message: &Fetch) {
    let lines: Vec<&str> = str::from_utf8(message.rfc822().unwrap())
        .unwrap()
        .split("\n")
        .collect();

    let mut last_mail = MAIL_LOCK.write().unwrap();
    last_mail.clear();
    let mut mimepart: bool = false;
    for x in lines.iter() {
        if x.starts_with("Date: ") {
            *date = x.to_string();
        } else if x.starts_with("From: ") {
            *from = x.to_string();
        } else if x.starts_with("Subject: ") {
            *subject = x.to_string();
        } else if x.starts_with("--") {
            if mimepart {
                break;
            };
            mimepart = true;
        } else if mimepart {
            if !x.starts_with("Content-") && !x.is_empty() {
                last_mail.push_str(x);
            }
        }
    }
    from.pop();
    subject.pop();
    date.pop();
    *from = from.replace("From: ", "");
    *subject = subject.replace("Subject: ", "");
    *date = date.replace(" +0000", "");
    *date = date.replace("Date: ", "");
}

/// Assign a user with given the mail number and the user
pub fn assign(args: &Vec<&str>) -> String {
    let mail_id = args[1].parse::<u32>().unwrap();
    let user = args[2];

    let mut info = MAIL_INFO_FILE.write().unwrap();
    let index = match info
        .stored
        .unassigned
        .iter()
        .position(|(x, _)| *x == mail_id)
    {
        Some(index) => index,
        None => {
            return format!("Mail {} Was not found in unassigned list.", mail_id);
        }
    };
    let subject = info.stored.unassigned[index].1.clone();
    info.stored
        .assigned
        .push((mail_id, subject, String::from(user)));
    info.stored.unassigned.remove(index);
    info.write_stored().unwrap();
    return format!("Mail {} Was assigned to {}.", mail_id, user);
}

/// Resolve the mail with this given number
pub fn resolve(args: &Vec<&str>) -> String {
    let mail_id = args[1].parse::<u32>().unwrap();

    let mut info = MAIL_INFO_FILE.write().unwrap();
    let index = match info
        .stored
        .assigned
        .iter()
        .position(|(x, _, _)| *x == mail_id)
    {
        Some(index) => index,
        None => {
            return format!("Mail {} Was not found in assigned list.", mail_id);
        }
    };
    let resolved = info.stored.assigned.remove(index);
    info.stored.resolved.push(resolved);
    info.write_stored().unwrap();
    return format!("Mail {} Was resolved", mail_id);
}

pub fn content(_args: &Vec<&str>) -> String {
    MAIL_LOCK.read().unwrap().clone()
}

pub fn display_unassigned(_args: &Vec<&str>) -> String {
    MAIL_INFO_FILE.read().unwrap().stored.display_unassigned()
}

pub fn display_assigned(args: &Vec<&str>) -> String {
    if args.len() == 2 {
        MAIL_INFO_FILE
            .read()
            .unwrap()
            .stored
            .display_assigned(String::from(args[1]))
    } else {
        MAIL_INFO_FILE
            .read()
            .unwrap()
            .stored
            .display_assigned(String::from("all"))
    }
}

pub fn display_resolved(args: &Vec<&str>) -> String {
    if args.len() == 2 {
        MAIL_INFO_FILE
            .read()
            .unwrap()
            .stored
            .display_resolved(String::from(args[1]))
    } else {
        MAIL_INFO_FILE
            .read()
            .unwrap()
            .stored
            .display_resolved(String::from("all"))
    }
}

/// Main loop, that connect and retreive the mails.
pub fn check_mail() {
    let credentials = &CREDENTIALS_FILE.stored;
    if credentials.domain.is_empty() {
        println!("No credentials for mails");
        return;
    } else {
        println!("Check email thread started");
    }
    let domain: &str = &credentials.domain[..];
    let port = 993;
    let socket_addr = (domain, port);
    let ssl_connector = TlsConnector::builder().unwrap().build().unwrap();
    let mut imap_socket = Client::secure_connect(socket_addr, domain, &ssl_connector).unwrap();
    let mut mail_number;

    imap_socket
        .login(&*credentials.email, &*credentials.password)
        .unwrap();

    println!("Connected to mails");
    {
        let info = MAIL_INFO_FILE.write().unwrap();
        mail_number = info.stored.mail_number;
    }
    println!("Last session mail number: {}", mail_number);
    loop {
        thread::sleep(time::Duration::from_millis(1000 * 3600));
        let mailn_tmp;
        match imap_socket.select("INBOX") {
            Ok(mailbox) => {
                mailn_tmp = mailbox.exists;
                if mailn_tmp == 0 {
                    println!("Error: Getting 0 emails number, continuing execution");
                    continue;
                }
                println!("Mailbox emails number = {}", mailbox.exists);
            }
            Err(e) => {
                println!("Error selecting INBOX: {}", e);
                continue;
            }
        };

        let mut received_emails = Vec::new();
        for x in mail_number..mailn_tmp {
            let seq = (x + 1).to_string();
            println!("Fetching mail {}", seq);

            let (mut date, mut from, mut subject) = (String::new(), String::new(), String::new());
            match imap_socket.fetch(&seq, "RFC822") {
                Ok(messages) => for message in messages.iter() {
                    parse_rfc(&mut date, &mut from, &mut subject, message);
                },
                Err(e) => println!("Error Fetching email {}: {}", seq, e),
            };

            let message = format!(
                "{} Email reçu le {} : {}, envoyé par {}",
                seq, date, subject, from
            );
            CHANNEL_MAILS.send_message(|m| m.content(message)).unwrap();
            received_emails.push((x + 1, (from, subject, date)));
        }
        mail_number = mailn_tmp;
        {
            let mut info = MAIL_INFO_FILE.write().unwrap();
            info.stored.mail_number = mail_number;
            info.stored.unassigned.append(&mut received_emails);
            info.write_stored().unwrap();
        }
    }
    // imap_socket.logout().unwrap();
}
