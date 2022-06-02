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

fn serial_to_mqtt(port_ref: Arc<Mutex<Box<dyn serialport::SerialPort>>>, txi: Sender<MqttStrMessage>) {
    loop {
        let mut serial_buf: Vec<u8> = vec![0; 16384];
        let mut previous_remainder = String::new();
        if let Ok(mut port) = port_ref.lock() {  
            let result = port.read(serial_buf.as_mut_slice());
            match { result } {
                Ok(m) =>  {
                    if m > 0 {
                        let mut start_idx = 0;

                        while start_idx < m {
                            let mut idx = start_idx;

                            while idx < m && serial_buf[idx] as char != '\n' {
                                idx = idx+1
                            }

                            let end_idx = idx;
                            let str_msg = str::from_utf8(&serial_buf[start_idx..end_idx]).expect("String parsing failed");
                            if idx < m {
                                log::debug!("Handing data over for mqtt send {:?}", str_msg);
                                let msg = MqttStrMessage {topic: "test".to_string(), payload: previous_remainder+str_msg};
                                previous_remainder = String::new();
                                if let Err(m) = txi.send(msg) {
                                    log::error!("On push to mqqt thread: {:?}", m);
                                }
                            } else {
                                previous_remainder = str_msg.to_string();
                            }
                            start_idx = end_idx+2; //\n is always followed by \r, skip both
                        }
                    }
                },
                Err(_m) => (),
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}


fn main() {
    env_logger::init();
    let (tx, rx) = launch_mqtt("localhost".to_string(), 1883, vec!["test_topic".to_string()], "".to_string());
  
    let port_ref = Arc::new(Mutex::new(serialport::new("/dev/ttyACM1", 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port")));

    /*let output = "{\"a\": 0}?".as_bytes();
    {
        if let Ok(mut x) = port_ref.lock() {
            x.write(output).expect("Write failed!");
        }
    }*/

    thread::sleep(Duration::from_millis(1000));
    let cloned_port_ref = port_ref.clone();
    spawn(move || serial_to_mqtt(cloned_port_ref, tx)); 


    loop {
        let msg = rx.recv();
        match msg {
            Ok(m) => {
                println!("{}", m.payload);
                if let Ok(mut port) = port_ref.lock() {  
                    port.write(m.payload.as_bytes()).expect("Write failed!");
                    port.write("?".as_bytes()).expect("Write failed");
                }
            },
            Err(e) => log::error!("Mqtt publish channel: {:?}", e),
        }
    }
    /*loop {
        let mut serial_buf: Vec<u8> = vec![0; 1024];

        if let Ok(mut x) = port.lock() {
            let result = x.read(serial_buf.as_mut_slice());

            match { result } {
                Ok(m) =>  {
                    println!("{} {}", m, String::from_utf8(serial_buf[0..m].to_vec()).expect("String parsing failed"));
                },
                Err(_m) => (),
            }
        }
    }*/

    
    


    /*let mut serial_buf: Vec<u8> = vec![0; 32];
    port.read(serial_buf.as_mut_slice()).expect("Found no data!");
    println!("{}", serial_buf.len())*/
    /*loop {
        let received = mqtt_messenger.receive_messages();
        for x in received {
            println!("{}", x.payload)
        }
    }*/
}
