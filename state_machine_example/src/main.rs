use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::io::{AsyncWriteExt,AsyncReadExt};
use tokio::net::TcpListener;
use std::thread;
use std::thread::sleep;
use std::time;
use rand::Rng;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Message {
    StateChangingTo(String),
    FailedToChangeState,
}

pub trait State {
    fn new() -> Self;
    fn state_info() ->  &'static str;
}

#[derive(Debug,Clone)]
struct Init;
#[derive(Debug,Clone)]
struct Downloading;
#[derive(Debug,Clone)]
struct Compressing;
#[derive(Debug,Clone)]
struct PushRemote;
#[derive(Debug,Clone)]
struct Completed;


impl State for Init {
    fn new() -> Self {
        Init
    }
    fn state_info() -> &'static str {
        "Init"
    }
}

impl State for Downloading {
    fn new() -> Self {
        Downloading
    }
    fn state_info() ->  &'static str {
        "Downloading"
    }
}

impl State for Compressing {
    fn new() -> Self {
        Compressing
    }
    fn state_info() ->  &'static str {
        "Compressing"
    }
}

impl State for PushRemote {
    fn new() -> Self {
        PushRemote
    }
    fn state_info() ->  &'static str {
        "PushRemote"
    }
}

impl State for Completed {
    fn new() -> Self {
        Completed
    }
    fn state_info() ->  &'static str {
        "Completed"
    }
}

#[derive(Debug,Clone)]
pub struct PushWorker<S : State> {
    download_urls : Vec<String>,
    downloaded_counts: usize,
    is_compress: bool,
    is_pushed: bool,
    state: S,
}

impl<S: State> PushWorker<S> {
    pub fn new(download_urls: Vec<String>, state: S) -> Self {
        Self {
            download_urls,
            downloaded_counts: 0,
            is_compress: false,
            is_pushed: false,
            state,
        }
    }

}

impl PushWorker<Init> {
    pub fn print_info(&self) {
        println!("State machine is Initiated waiting for all machine to setup");
    }
}

impl PushWorker<Downloading> {
    pub fn downloading_files(&mut self) {
        println!("downloading files");
        for _ in &self.download_urls {
            let mut rng = rand::thread_rng();
            let random_number: u64 = rng.gen_range(1,10);
        
            sleep(time::Duration::from_secs(random_number));
            self.downloaded_counts += 1;
        }
    }
}

impl PushWorker<Compressing> {
    pub fn compressing_files(&mut self) {
        let mut rng = rand::thread_rng();
        let random_number: u64 = rng.gen_range(1,10);
        sleep(time::Duration::from_secs(random_number));
        self.is_compress = true;
    }
}

impl PushWorker<PushRemote> {
    pub fn pushing_files(&mut self) {
        let mut rng = rand::thread_rng();
        let random_number: u64 = rng.gen_range(1,10);
        sleep(time::Duration::from_secs(random_number));
        self.is_pushed = true;
    }
}

impl PushWorker<Completed> {
    pub fn greeter(&self) {
        println!("task completed");
    }
}


pub fn move_state<S: State, T: State>(worker: PushWorker<S>) -> Result<PushWorker<T>, String> {

    if S::state_info() == "Init" {
        println!("State machine is changing state from Init to Downloading");
        Ok(PushWorker {
            download_urls: worker.download_urls,
            downloaded_counts: worker.downloaded_counts,
            is_compress: worker.is_compress,
            is_pushed: worker.is_pushed,
            state: T::new(),
        })
    }else if S::state_info() == "Downloading" && worker.download_urls.len() == worker.downloaded_counts {
        println!("State machine is changing state from Downloading to Compressing");
        Ok(PushWorker {
            download_urls: worker.download_urls,
            downloaded_counts: worker.downloaded_counts,
            is_compress: worker.is_compress,
            is_pushed: worker.is_pushed,
            state: T::new(),
        })
    } else if S::state_info() == "Compressing" && worker.is_compress {
        println!("State machine is changing state from Compressing to PushRemote");
        Ok(PushWorker {
            download_urls: worker.download_urls,
            downloaded_counts: worker.downloaded_counts,
            is_compress: worker.is_compress,
            is_pushed: worker.is_pushed,
            state: T::new(),
        })
    }else if S::state_info() == "PushRemote" && worker.is_pushed {
        println!("State machine is changing state from PushRemote to Completed");
        Ok(PushWorker {
            download_urls: worker.download_urls,
            downloaded_counts: worker.downloaded_counts,
            is_compress: worker.is_compress,
            is_pushed: worker.is_pushed,
            state: T::new(),
        })
    } else {
        Err(String::from("Transition invalid"))
    }
}


#[tokio::main]
async fn main() {

    //flag for indicating state change
    let state_change_flag = Arc::new(Mutex::new(false));

    //address of nodes
    let mut network_config = vec!["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"];

    // reading node id
    println!("Enter your id:");
    let mut node_id = String::new();
    std::io::stdin().read_line(&mut node_id).expect("Failed to read from stdin");
    let node_id:usize = node_id.trim().parse().unwrap();
    
    //fetching own address
    let my_address:&str = network_config.get(node_id).unwrap();
    
    //vector for storing incoming messages temporary
    let received_messages =  Arc::new(Mutex::new(Vec::new()));
    let rec_msg_clone =  Arc::clone(&received_messages);
    let state_flag_clone = Arc::clone(&state_change_flag);

    //spawing message listener
    let _listener_spawn = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind(my_address).await.unwrap();
            println!("Server listening on {}", my_address);
            while let Ok((socket, _)) = listener.accept().await {
                let received_messages = Arc::clone(&rec_msg_clone);
                let flag_clone = Arc::clone(&state_flag_clone);
                tokio::spawn(async move {
                    handle_client(socket, received_messages, flag_clone).await;
                });
            }
        });
    });

    println!("waiting for listener to start");
    //additional sleep so that we can start other nodes in other terminals
    thread::sleep(time::Duration::from_secs(15));

    //removing own address and connecting to all other address in config
    network_config.remove(node_id);
    println!("connecting to other nodes");
    //storing writing half
    let mut writers: Vec<_> = Vec::new();
    for writer in  network_config {
        let stream = TcpStream::connect(writer).await.unwrap();
        writers.push(tokio::io::split(stream).1);
    }
    
    
    //initiating push worker
    let download_urls = vec![String::from("www.w1.com"), String::from("www.w2.com")];
    let pushworker = PushWorker::new(download_urls, Init);
    pushworker.print_info();

    sleep(Duration::from_secs(5));
    // Init
    //sending state change msg
    let msg = Message::StateChangingTo(String::from("Downloading"));
    send_message_to_all_nodes(&mut writers, msg).await;
    while *state_change_flag.lock().unwrap() != true {
        sleep(Duration::from_secs(1));
    }
    
    //Downloading
    let mut pushworker  = move_state::<Init, Downloading>(pushworker).unwrap();
    *state_change_flag.lock().unwrap() = false;
    pushworker.downloading_files();
    //sending state change msg
    let msg = Message::StateChangingTo(String::from("Compressing"));
    send_message_to_all_nodes(&mut writers, msg).await;
    //wait untill we get flag to change state
    while *state_change_flag.lock().unwrap() != true {
        sleep(Duration::from_secs(1));
    }
    
    //Compressing
    let mut pushworker  = move_state::<Downloading, Compressing>(pushworker).unwrap();
    *state_change_flag.lock().unwrap() = false;
    pushworker.compressing_files();
    // sending state change msg
    let msg = Message::StateChangingTo(String::from("PushRemote"));
    send_message_to_all_nodes(&mut writers, msg).await;
    // wait untill we get flag to change state
    while *state_change_flag.lock().unwrap() != true {
        sleep(Duration::from_secs(1));
    }

    //Pushremote
    let mut pushworker = move_state::<Compressing, PushRemote>(pushworker).unwrap();
    *state_change_flag.lock().unwrap() = false;
    pushworker.pushing_files();
    //sending state change msg
    let msg = Message::StateChangingTo(String::from("Completed"));
    send_message_to_all_nodes(&mut writers, msg).await;
    //wait untill we get flag to change state
    while *state_change_flag.lock().unwrap() != true {
        sleep(Duration::from_secs(1));
    }

    //Completed
    let pushworker = move_state::<PushRemote, Completed>(pushworker).unwrap();
    pushworker.greeter();

}

//for handling incoming messages
async fn handle_client(socket: TcpStream, received_messages: Arc<Mutex<Vec<Message>>>, flag : Arc<Mutex<bool>>) {
    let mut reader = tokio::io::split(socket).0;
    let mut buffer = [0; 1024];

    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break, // Connection closed
            Ok(bytes_read) => {
                let message: Message = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
                let mut messages = received_messages.lock().unwrap();
                println!("{:?}", &message);
                messages.push(message);

                if messages.len() == 2 {
                    *flag.lock().unwrap() = true;
                    messages.drain(..);
                }
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}


//for sending messages to other nodes
async fn send_message(mut writer: &mut tokio::io::WriteHalf<TcpStream>, msg: Message) {
    let serialized = serde_json::to_vec(&msg).expect("Failed to serialize message");
    writer.write_all(&serialized).await.expect("Failed to write to socket");
    writer.flush().await.expect("Failed to flush socket");
}

async fn send_message_to_all_nodes(writers: &mut Vec<tokio::io::WriteHalf<TcpStream>>, msg : Message) {
    for writer in writers {
        send_message(writer, msg.clone()).await;
    }
}