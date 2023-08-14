#![feature(async_closure)]
use clap::Parser;
use hashbag::HashBag;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use scraper::{Html, Selector};
use std::{fs, path::Path, thread::sleep, time::Duration};
use url::Url;

#[derive(Parser, Debug)]
#[clap(author="Luka Pršina", version="0.1.0", about="Download musescore scores as svg files", long_about = None)]
struct Args {
    url: String,
    #[clap(short, long)]
    div_class: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new("output");

    if output_path.exists() {
        if output_path.read_dir()?.next().is_some() {
            panic!("Output directory already exists!");
        }
    } else {
        fs::create_dir("output")?;
    }

    let args = Args::parse();
    let div_class = args.div_class.unwrap_or("vAVs3".to_string());
    let site = &args.url;

    println!("Starting a new web browser\nSite: {}", site);
    let sources = get_sources(site, &div_class).await?;

    println!("\nFound {} pages, saving into ./output", sources.len());
    let mut titles: HashBag<String> = HashBag::new();

    for (pos, source) in sources.iter().enumerate() {
        let response = reqwest::get(source).await?;
        let body = response.bytes().await?;
        let url = Url::parse(source)?;

        let title = match url.path_segments() {
            Some(segments) => match segments.last() {
                Some(last) => {
                    let count = titles.insert(last.to_string());
                    let path = Path::new(last);
                    if count != 0 {
                        let stem = path.file_stem().unwrap_or_default();
                        let extension = path.extension().unwrap_or_default();
                        format!(
                            "{}-{}.{}",
                            stem.to_str().expect("Can't convert URL title to string"),
                            count,
                            extension
                                .to_str()
                                .expect("Can't convert URL extension to string")
                        )
                    } else {
                        last.to_string()
                    }
                }
                None => format!("img{}", pos),
            },

            None => format!("img{}", pos),
        };

        fs::write(&format!("output/{}", title), body).expect("Can't write file");
    }

    Ok(())
}

async fn get_sources(
    site: &str,
    div_class: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let options = LaunchOptionsBuilder::default()
        .headless(false)
        .idle_browser_timeout(Duration::from_secs(600))
        .build()?;
    let browser = Browser::new(options)?;
    let mut sources: Vec<String> = vec![];

    let tab = browser.wait_for_initial_tab()?;

    tab.navigate_to(site)?
        .wait_until_navigated()
        .expect("Can't open site");

    let handle = tokio::spawn(async move {
        let prompt = inquire::Confirm::new("Press enter when logged in");
        let _ = prompt.prompt();
    });

    handle.await?;

    let divs = tab
        .wait_for_elements(&format!("div.{}", div_class))
        .unwrap();

    for (pos, div) in divs.iter().enumerate() {
        div.scroll_into_view()?;

        println!("Sleeping");
        sleep(Duration::from_secs(1));

        println!("Getting page #{}", pos + 1);

        let html = div
            .call_js_fn("function() { return this.innerHTML;}", false)?
            .value
            .expect("Can't get innerHTML on div");

        let text = html.as_str().expect("Can't convert HTML to string");
        let fragment = Html::parse_fragment(text);
        let selector = Selector::parse("img").expect("Wrong html selector");
        let mut img = fragment.select(&selector);

        let src = img
            .next()
            .expect("No image element on page")
            .value()
            .attr("src")
            .expect("No src attribute on image element")
            .to_string();

        if img.next().is_some() {
            println!(
                "Found more than one image in one page, skipping.\nThis is not supported yet."
            );
        }
        sources.push(src);
    }

    Ok(sources)
}
