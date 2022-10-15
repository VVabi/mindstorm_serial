mod mqtt_wrapper;
mod library;

use mqtt_wrapper::mqtt_thread::launch_mqtt;
use library::types::MqttStrMessage;
use std::time::Duration;
use std::thread;
use String;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::str;
use std::thread::spawn;
use serde_json::Value;

fn send_via_mqtt(previous_remainder: String, bytes_read: usize, serial_buf: &Vec<u8>, tx: &Sender<MqttStrMessage>) -> String {
    let mut start_idx = 0;
    let mut remainder = previous_remainder;
    while start_idx < bytes_read {
        let mut idx = start_idx;

        while idx < bytes_read && serial_buf[idx] as char != '\n' {
            idx = idx+1
        }

        let end_idx = idx;
        let str_msg = str::from_utf8(&serial_buf[start_idx..end_idx]).expect("String parsing failed");
        if idx < bytes_read {
            let full_payload = remainder+str_msg;
            remainder = String::new();
            log::debug!("Handing data over for mqtt send {:?}", full_payload);
            let json_conversion_result : std::result::Result<Value, serde_json::Error> = serde_json::from_str(&(full_payload));
            match json_conversion_result {
                Ok(v) => {
                    let topic = match &v["topic"] {
                        Value::String(c) => c.clone(),
                        _ => unreachable!()
                    };
                    let msg = MqttStrMessage {topic: topic, payload: v["payload"].to_string() };
                    
                    if let Err(m) = tx.send(msg) {
                        log::error!("On push to mqtt thread: {:?}", m);
                    }
                },
                Err(m) => log::error!("Json conversion failed: {:?} {}", m, full_payload),
            }
        } else {
            remainder = str_msg.to_string();
        }
        start_idx = end_idx+2; //\n is always followed by \r, skip both
   }

   return remainder
}


fn serial_to_mqtt(port_ref: Arc<Mutex<Box<dyn serialport::SerialPort>>>, txi: Sender<MqttStrMessage>) {
    let mut remainder = String::new();
    let mut serial_buf: Vec<u8> = vec![0; 16384];

    loop {
        let mut bytes_read = 0;
        if let Ok(mut port) = port_ref.lock() {  
            let result = port.read(serial_buf.as_mut_slice());
            match { result } {
                Ok(m) =>  bytes_read = m,
                Err(_e) => ()
            }
        }
        if bytes_read > 0 {
            remainder = send_via_mqtt(remainder, bytes_read, &serial_buf, &txi);
        }

        thread::sleep(Duration::from_millis(10));
    }
}


fn main() {
    env_logger::init();
    let (tx, rx) = launch_mqtt("localhost".to_string(), 1883, vec!["motor/register".to_string(), "motor/set_pwm".to_string(), "motor/goto_position".to_string()], "".to_string());
    
    let mut raw_port = serialport::new("/dev/ttyACM0", 115_200)
    .timeout(Duration::from_millis(10))
    .open().expect("Failed to open port");

    // Flush serial port to get clean state (the opposing side must handle this empty message gracefully!)
    raw_port.write("{}?".as_bytes()).expect("Flush failed");
    let port_ref = Arc::new(Mutex::new(raw_port));


    thread::sleep(Duration::from_millis(1000));
    let cloned_port_ref = port_ref.clone();
    spawn(move || serial_to_mqtt(cloned_port_ref, tx)); 


    loop {
        let msg = rx.recv();
        match msg {
            Ok(m) => {
                println!("{}", m.payload);
                if let Ok(mut port) = port_ref.lock() {
                    println!("{}", m.payload);
                    let msg = format!(r#"{{"topic": "{}","payload":  {} }}?"#, m.topic, m.payload);
                    println!("{}", &msg);
                    port.write(msg.as_bytes()).expect("Write failed");
                }
            },
            Err(e) => log::error!("Mqtt publish channel: {:?}", e),
        }
    }
}
