use crate::clippings::{Clipping, ClippingKind};

pub trait Display {
    fn to_org_text(&self) -> String;
}

impl Display for Clipping {
    fn to_org_text(&self) -> String {
        let mut org_text = String::new();
        org_text.push_str("#+begin_quote\n");
        if self.kind == ClippingKind::Note {
            org_text.push_str(&format!("<{}>", self.content))
        } else {
            org_text.push_str(&self.context_aware_formatted_content());
        }
        org_text.push_str("\n#+end_quote");
        org_text
    }
}
