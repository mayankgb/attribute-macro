use std::io::Error;
use macro_traits::{Serialize, Deserialize};
use macros::{Serialize, DeserializeStruct};
use attribute_macro::MySerde;


#[derive(Serialize, DeserializeStruct, Debug)]
struct Swap { 
    base_asset: String, 
    base_qty: u32, 
    quote_asset: String, 
    quote_qty: u32
}

#[derive(MySerde)]
struct Person { 
    #[serde(skip_serializing_if = "Option::is_none", rename = "username")]
    name: Option<String>,
    email: String, 
    password: String
    
}

fn main() {

    let swap = Swap { 
        base_asset: String::from("USDC"), 
        quote_asset: String::from("BTC"), 
        base_qty: 200, 
        quote_qty: 300
    }; 
    
    let person = Person { 
        name: Some(String::from("mayank")),
        password: String::from("value"), 
        email: String::from("value")
    };

    let re = person.json();
    let swap_bytes = swap.serialize(); 
    let result = Swap::deserialize(&swap_bytes).unwrap();


    println!("swap bytes {:?}", swap_bytes);
    println!("{:?}", re);
    println!("swap result is {:?}", result);
}