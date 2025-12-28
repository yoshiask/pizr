use calcard::vcard::{VCardParameterName, VCardProperty};

pub struct UserContact {
    pub formatted_name: Option<String>,
    pub perferred_tel: Option<String>,
    pub perferred_email: Option<String>,
}

impl UserContact {
    fn new() -> UserContact {
        UserContact {
            formatted_name: None,
            perferred_tel: None,
            perferred_email: None,
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

            VCardProperty::Tel => {
                if entry.params.iter().any(|p| p.name == VCardParameterName::Pref) || vcard.perferred_tel.is_none() {
                    vcard.perferred_tel = Some(entry.values.first()?.as_text()?.to_string());
                }
            }

            VCardProperty::Email => {
                if entry.params.iter().any(|p| p.name == VCardParameterName::Pref) || vcard.perferred_email.is_none() {
                    vcard.perferred_email = Some(entry.values.first()?.as_text()?.to_string());
                }
            }

            _ => {}
        }
    }

    Some(vcard)
}