use clap::Parser;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use scraper::{Html, Selector};
use std::{fs, path::Path, thread::sleep, time::Duration};

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
    let div_class = args.div_class.unwrap_or_else(|| "vAVs3".to_string());
    let site = &args.url;

    println!("Starting a new web browser\nSite: {}", site);
    let sources = get_sources(site, &div_class)?;

    println!("\nFound {} pages, saving into ./output", sources.len());

    for (pos, source) in sources.iter().enumerate() {
        let body = reqwest::get(source).await?.text().await?;
        fs::write(&format!("output/img{}.svg", pos), body).unwrap();
    }

    Ok(())
}

fn get_sources(site: &str, div_class: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let options = LaunchOptionsBuilder::default().headless(false).build()?;
    let browser = Browser::new(options)?;
    let mut sources: Vec<String> = vec![];

    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to(site)?.wait_until_navigated().unwrap();
    let divs = tab.wait_for_elements(&format!("div.{}", div_class))?;
    sleep(Duration::from_secs(1));

    for (pos, div) in divs.iter().enumerate() {
        div.scroll_into_view()?;

        println!("Sleeping");
        sleep(Duration::from_secs(1));

        println!("Getting page #{}", pos + 1);

        let html = div
            .call_js_fn("function() { return this.innerHTML;}", false)?
            .value
            .unwrap();

        let text = html.as_str().unwrap();
        let fragment = Html::parse_fragment(text);
        let selector = Selector::parse("img").unwrap();
        let mut img = fragment.select(&selector);

        let src = img.next().unwrap().value().attr("src").unwrap().to_string();
        if img.next().is_some() {
            println!(
                "Found more than one image in one page, skipping.\nThis is not supported yet."
            );
        }
        sources.push(src);
    }

    Ok(sources)
}
