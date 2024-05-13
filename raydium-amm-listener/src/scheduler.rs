// task_manager.rs
use futures::executor::block_on;
use futures::stream::StreamExt;
use futures::{channel::mpsc, SinkExt};
use std::sync::{Arc, Mutex};

pub struct TaskManager {
    sender: Arc<Mutex<mpsc::Sender<String>>>,
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    tasks: Arc<Mutex<Vec<String>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let sender = Arc::new(Mutex::new(sender));
        let receiver = Arc::new(Mutex::new(receiver));
        let tasks = Arc::new(Mutex::new(Vec::new()));
        TaskManager {
            sender,
            receiver: receiver.clone(),
            tasks: tasks.clone(),
        }
    }

    pub fn add_task(&self, task: String) {
        let mut sender = self.sender.clone();
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            let testing = task.clone();
            sender.lock().unwrap().send(testing.to_string()).await; //.await.unwrap();
                                                                    // sender.send(task.clone()).await.unwrap();
                                                                    // tasks.lock().unwrap().push(task);
        });
    }

    pub async fn run(&self) {
        // let mut receiver = self.receiver.clone();

        println!("Starting task manager");
        // block_on(async move {\
        // let shared_self = Arc::new(self);
        while let Some(task) = self.receiver.lock().unwrap().next().await {
            let tasks = self.tasks.clone();

            let sender = self.sender.clone();
            // let testing = shared_self.clone();
            // let task = task.clone();
            tokio::spawn(async move {
                // testing.add_task(task.clone());/
                // let testing = shared_self.clone();
                println!("Start Executing task: {}", task);
                // // Simulate task completion
                std::thread::sleep(std::time::Duration::from_secs(10));
                println!("Task completed: {}", task);

                // // self.add_task(task.clone());
                // // Add a new task when the current one is completed
                // // Remove the completed task from the task list
                tasks.lock().unwrap().retain(|t| t != &task);

                sender.lock().unwrap().send("Task completed".to_string());
                // .await;

                // let mut sender = self.sender.clone();
                // testing.add_task(format!("Follow-up task for {}", task));
            });
            // println!("Start Executing task: {}", task);
            // // Simulate task completion
            // std::thread::sleep(std::time::Duration::from_secs(10));
            // println!("Task completed: {}", task);

            // self.add_task(task.clone());
            // // Add a new task when the current one is completed
            // // self.add_task(format!("Follow-up task for {}", task));
            // // Remove the completed task from the task list
            // tasks.lock().unwrap().retain(|t| t != &task);
        }
        // });
    }
}
