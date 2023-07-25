pub mod weather_data {
    
    use reqwest;
    use xml::reader::{EventReader, XmlEvent};
    use rusqlite::{Connection, Result, OpenFlags};
    
    #[derive(Debug)]
    pub struct Observation {
        name: String,
        value: f32,
        unit: String
    }
    
    pub async fn obtain_data(url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        
        let response = reqwest::get(url).await?.bytes().await?;
        
        let xml_reader = EventReader::new(response.as_ref());
        
        let mut obs_flag = false;
        let mut start_flag = false;
        let mut what_to_write = -1;
        let mut params: Vec<String> = Vec::new();
        
        for e in xml_reader{
            match e {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    if name.local_name == "observations"{
                        obs_flag = true;
                    }
                    if start_flag == true {
                        if name.local_name == "date_time" || name.local_name == "observation_name" || name.local_name == "observation_value" || name.local_name == "observation_unit" {
                            what_to_write = 1;
                        }
                        else {
                            what_to_write = -1;
                        }
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    if name.local_name == "observations"{
                        obs_flag = false;
                        if start_flag == true {
                            break;
                        }    
                    }

                }
                Ok(XmlEvent::Characters(text)) => {
                    if obs_flag == true && text == "LEFKOSIA" {
                        start_flag = true;
                    } else if start_flag == true {                           
                        if what_to_write == 0 || what_to_write == 1{
                            params.push(text)
                        }
                    }
                }
                
                Err(e) => {
                    return Err(Box::new(e));
                }
                _ => {}
            }
        }
        Ok(params)

    }
    
    
    pub fn store_data (data: Vec<String>) -> Result<(), Box<dyn std::error::Error>>  {
        
        //first value is date time
        let str_date_time = data[0].split('(').next().unwrap().to_string();
        
        //from string to observation
        let mut obs: Vec<Observation> = Vec::new();
        {
            let mut count = 0;
            let mut index = 0;
            for elem in &data[1..] {
                match count % 3 {
                    0 => obs.push(Observation { name: elem.to_string(), value: 0.0, unit: String::new() }),
                    1 => obs[index].value = elem.parse().expect("Failed to parse string as float"),
                    2 => { obs[index].unit = elem.to_string(); index+=1;}
                    _ => {}
                }
                count += 1;
            }
        }
        //create db if not exists
        let db_path = String::from("./weather_data.db");
        let db_connection =  Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;
            
        //create table if not exists
        let db_create_table = String::from("CREATE TABLE IF NOT EXISTS weather_data (
            id INTEGER PRIMARY KEY,
            dt TEXT NOT NULL,
            city TEXT NOT NULL,
            name TEXT NOT NULL,
            value INTEGER NOT NULL,
            unit TEXT NOT NULL
            )"
        );
        db_connection.execute(&db_create_table, [])?;

        //insert new values in the table
        let db_insert_table = String::from("insert into weather_data (dt, city, name, value, unit) values (?1, ?2, ?3, ?4, ?5)");
        for elem in obs {
            db_connection.execute(&db_insert_table, [str_date_time.clone(), "Lefcosia".to_string(), elem.name, elem.value.to_string(), elem.unit])?;    
        }
        Ok(())

    }

}