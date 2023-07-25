
mod weather_data;

use weather_data::{weather_data::obtain_data, weather_data::store_data};

#[tokio::main]
async fn  main() {
    let params = obtain_data("https://www.dom.org.cy/AWS/OpenData/CyDoM.xml").await.expect("Cannot obtain or parse data\n");

    match store_data(params) {
        Ok(()) => (),
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };
}
