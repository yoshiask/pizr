use calcard::vcard::{VCardProperty};

pub struct UserContact {
    pub formatted_name: Option<String>,
}

impl UserContact {
    fn new() -> UserContact {
        UserContact {
            formatted_name: None
        }
    }
}

pub fn parse_vcard_from_file(filename: &str) -> Option<UserContact> {
    let vcard_contents = std::fs::read_to_string(filename).ok()?;
    let vcard_data = calcard::vcard::VCard::parse(vcard_contents).ok()?;
    let mut vcard = UserContact::new();

    for entry in vcard_data.entries.iter() {
        match entry.name {
            VCardProperty::Fn => vcard.formatted_name = Some(entry.values.first()?.as_text()?.to_string()),

            _ => {}
        }
    }

    Some(vcard)
}