use std::sync::Arc;

use anyhow::{anyhow, Result};
use derive_typst_intoval::{IntoDict, IntoValue};
use generic_odrl::generic_action::Action;
use generic_odrl::generic_asset::Asset;
use generic_odrl::generic_party::Party;
use generic_odrl::generics::StringOrX;
use serde::{Deserialize, Serialize};
use typst::foundations::{Bytes, Dict, IntoValue};
use typst::text::Font;
use typst_as_lib::TypstTemplate;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

static TEMPLATE_FILE: &str = include_str!("../templates/template.typ");
static FONT_BLACK: &[u8] = include_bytes!("../fonts/Roboto-Black.ttf");
static FONT_BLACK_ITALIC: &[u8] = include_bytes!("../fonts/Roboto-BlackItalic.ttf");
static FONT_BOLD: &[u8] = include_bytes!("../fonts/Roboto-Bold.ttf");
static FONT_BOLD_ITALIC: &[u8] = include_bytes!("../fonts/Roboto-BoldItalic.ttf");
static FONT_ITALIC: &[u8] = include_bytes!("../fonts/Roboto-Italic.ttf");
static FONT_LIGHT: &[u8] = include_bytes!("../fonts/Roboto-Light.ttf");
static FONT_LIGHT_ITALIC: &[u8] = include_bytes!("../fonts/Roboto-LightItalic.ttf");
static FONT_MEDIUM: &[u8] = include_bytes!("../fonts/Roboto-Medium.ttf");
static FONT_MEDIUM_ITALIC: &[u8] = include_bytes!("../fonts/Roboto-MediumItalic.ttf");
static FONT_REGULAR: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
static FONT_THIN: &[u8] = include_bytes!("../fonts/Roboto-Thin.ttf");
static FONT_THIN_ITALIC: &[u8] = include_bytes!("../fonts/Roboto-ThinItalic.ttf");
//static CC_BY: &str = include_str!("../templates/cc/by-sa.typ");

// Implement Into<Dict> manually, so we can just pass the struct
// to the compile function.
impl From<Content> for Dict {
    fn from(value: Content) -> Self {
        value.into_dict()
    }
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Content {
    v: Vec<ContractTerms>,
    assigner: String,
    assignee: String,
    asset: String,
    odrl: String,
    //cc: Option<String>,
}

#[derive(Debug, Clone, Default, IntoValue, IntoDict)]
pub struct ContractTerms {
    pub heading: String,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct Templates {
    clauses: Vec<Template>,
}

#[derive(Serialize, Deserialize)]
pub enum Variant {
    Permission,
    Prohibition,
    Duty,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    heading: Option<String>,
    key: String,
    clause: Option<String>,
    definitions: Option<Vec<String>>,
    required: bool,
    #[serde(rename = "type")]
    variant: String,
}

pub(crate) fn load_templates() -> Result<Vec<Template>> {
    let templates =
        serde_json::from_str::<Templates>(include_str!("../templates/buildingblocks.json"))?;

    Ok(templates.clauses)
}

fn load_fonts() -> Result<Vec<Font>> {
    let mut fonts = Vec::new();
    fonts.push(
        Font::new(Bytes::from(FONT_BLACK), 0).ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_BLACK_ITALIC), 0)
            .ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts
        .push(Font::new(Bytes::from(FONT_BOLD), 0).ok_or_else(|| anyhow!("Unable to query Font"))?);
    fonts.push(
        Font::new(Bytes::from(FONT_BOLD_ITALIC), 0)
            .ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_ITALIC), 0).ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_LIGHT), 0).ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_LIGHT_ITALIC), 0)
            .ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_MEDIUM), 0).ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_MEDIUM_ITALIC), 0)
            .ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts.push(
        Font::new(Bytes::from(FONT_REGULAR), 0).ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    fonts
        .push(Font::new(Bytes::from(FONT_THIN), 0).ok_or_else(|| anyhow!("Unable to query Font"))?);
    fonts.push(
        Font::new(Bytes::from(FONT_THIN_ITALIC), 0)
            .ok_or_else(|| anyhow!("Unable to query Font"))?,
    );
    Ok(fonts)
}

pub fn render_pdf(
    policy: generic_odrl::policy::GenericPolicy,
    blocks: Arc<Vec<Template>>,
) -> Result<Vec<u8>> {
    // Read in fonts and the main source file.
    // We can use this template more than once, if needed (Possibly
    // with different input each time).
    let template = TypstTemplate::new(load_fonts()?, TEMPLATE_FILE);

    let mut definitions = vec![];

    let terms = blocks
        .iter()
        .filter_map(|tmpl| {
            if tmpl.required {
                if tmpl.key == "BaseDefinitions" {
                    definitions.extend(tmpl.definitions.clone().unwrap_or_default());
                    None
                } else {
                    Some(ContractTerms {
                        heading: tmpl.heading.clone().unwrap_or_default(),
                        text: tmpl.clause.clone().unwrap_or_default(),
                    })
                }
            } else {
                match tmpl.variant.as_str() {
                    "permission" => {
                        if policy
                            .permission
                            .clone()
                            .unwrap_or_default()
                            .iter()
                            .find(|permission| {
                                get_string_from_action(&permission.action) == Some(tmpl.key.clone())
                            })
                            .is_some()
                        {
                            Some(ContractTerms {
                                heading: tmpl.heading.clone().unwrap_or_default(),
                                text: tmpl.clause.clone().unwrap_or_default(),
                            })
                        } else {
                            None
                        }
                    }
                    "prohibition" => {
                        if policy
                            .prohibition
                            .clone()
                            .unwrap_or_default()
                            .iter()
                            .find(|permission| {
                                get_string_from_action(&permission.action) == Some(tmpl.key.clone())
                            })
                            .is_some()
                        {
                            Some(ContractTerms {
                                heading: tmpl.heading.clone().unwrap_or_default(),
                                text: tmpl.clause.clone().unwrap_or_default(),
                            })
                        } else {
                            None
                        }
                    }
                    "duty" => {
                        if policy
                            .obligation
                            .clone()
                            .unwrap_or_default()
                            .iter()
                            .find(|permission| {
                                get_string_from_action(&permission.action) == Some(tmpl.key.clone())
                            })
                            .is_some()
                        {
                            Some(ContractTerms {
                                heading: tmpl.heading.clone().unwrap_or_default(),
                                text: tmpl.clause.clone().unwrap_or_default(),
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        })
        .collect::<Vec<_>>();

    let mut base_definitions = definitions.into_iter().fold(
        ContractTerms {
            heading: "Definitions".to_string(),
            text: "".to_string(),
        },
        |mut term, elem| {
            term.text += &format!("{elem}\n\n");
            term
        },
    );
    base_definitions.text = base_definitions
        .text
        .strip_suffix("\n\n")
        .unwrap_or_default()
        .to_string();

    let mut definitions_all = vec![base_definitions];
    definitions_all.extend(terms);

    let assignee = get_string_from_party(&policy.assignee).unwrap_or_default();
    let assigner = get_string_from_party(&policy.assigner).unwrap_or_default();
    let asset = get_string_from_asset(&policy.target).unwrap_or_default();

    let content = Content {
        v: definitions_all,
        assigner,
        assignee,
        asset,
        odrl: serde_json::to_string(&policy).unwrap(),
        //cc: None,
        //cc: Some(CC_BY.to_string()),
    };

    // Run it
    let doc = template.compile_with_input(content).output?;

    // Create pdf
    let mut options: PdfOptions<'_> = Default::default();
    options.standards = PdfStandards::new(&[PdfStandard::A_3b]).map_err(|e| anyhow!(e))?;

    Ok(typst_pdf::pdf(&doc, &options)
        .map_err(|e| anyhow!(format!("{:?} Unable to compile pdf", e)))?)
}

pub fn get_string_from_party(party: &Option<StringOrX<Party>>) -> Option<String> {
    match &party {
        Some(generic_odrl::generics::StringOrX::<Party>::String(string)) => Some(string.clone()),
        Some(generic_odrl::generics::StringOrX::<Party>::X(party)) => party.uid.clone(),
        _ => None,
    }
}

pub fn get_string_from_asset(party: &Option<StringOrX<Box<Asset>>>) -> Option<String> {
    match &party {
        Some(generic_odrl::generics::StringOrX::<Box<Asset>>::String(string)) => {
            Some(string.clone())
        }
        Some(generic_odrl::generics::StringOrX::<Box<Asset>>::X(asset)) => asset.uid.clone(),
        _ => None,
    }
}

pub fn get_string_from_action(action: &Option<StringOrX<Action>>) -> Option<String> {
    match &action {
        Some(generic_odrl::generics::StringOrX::<Action>::String(string)) => Some(string.clone()),
        Some(generic_odrl::generics::StringOrX::<Action>::X(asset)) => Some(asset.name.clone()),
        _ => None,
    }
}
