use anyhow::{anyhow, Result};
use derive_typst_intoval::{IntoDict, IntoValue};
use odrl::model::policy::AgreementPolicy;
use typst::foundations::{Bytes, Dict, IntoValue};
use typst::text::Font;
use typst_as_lib::TypstTemplate;

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
static CC_BY: &str = include_str!("../templates/cc/by-sa.typ");

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
    cc: Option<String>,
}

#[derive(Debug, Clone, Default, IntoValue, IntoDict)]
pub struct ContractTerms {
    pub heading: String,
    pub text: String,
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

pub fn render_pdf(policy: AgreementPolicy) -> Result<Vec<u8>> {
    // Read in fonts and the main source file.
    // We can use this template more than once, if needed (Possibly
    // with different input each time).
    let template = TypstTemplate::new(load_fonts()?, TEMPLATE_FILE);

    let assignee = policy
        .assignee
        .uid
        .as_ref()
        .ok_or_else(|| anyhow!("No asset found in policy"))?
        .clone();
    let assigner = policy
        .assigner
        .uid
        .as_ref()
        .ok_or_else(|| anyhow!("No asset found in policy"))?
        .clone();
    let asset = policy
        .target
        .as_ref()
        .ok_or_else(|| anyhow!("No asset found in policy"))?
        .uid
        .as_ref()
        .ok_or_else(|| anyhow!("No asset found in policy"))?
        .clone();

    let content = Content {
        v: vec![],
        assigner,
        assignee,
        asset,
        odrl: serde_json::to_string(&policy).unwrap(),
        //cc: None,
        cc: Some(CC_BY.to_string()),
    };

    // Run it
    let doc = template.compile_with_input(content).output?;

    // Create pdf
    let options = Default::default();

    let mut pdfbytes =
        typst_pdf::pdf(&doc, &options).map_err(|_| anyhow!("Unable to compile pdf"))?;

    pdfbytes.extend_from_slice(b"foobar");

    Ok(pdfbytes)
}
