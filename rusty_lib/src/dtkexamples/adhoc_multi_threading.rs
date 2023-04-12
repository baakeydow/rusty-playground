//! ad hoc multithreading example
use std::{
    sync::{Arc, Mutex},
    thread,
};

/// Task to be executed
#[derive(Default)]
#[allow(missing_docs)]
pub struct TaskSpawner<T> {
    pub id: u32,
    pub data: T,
    pub success: bool,
}

#[allow(missing_docs)]
impl<T: 'static + Default> TaskSpawner<T> {
    pub fn new(id: u32) -> Self {
        TaskSpawner {
            id,
            ..Default::default()
        }
    }
    pub fn set_task_data(&mut self, data: T) {
        self.data = data;
    }
    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }
}

/// Tasks Handler
pub struct Tasks<'a, T: std::marker::Send> {
    arc_data: &'a Arc<Mutex<Vec<TaskSpawner<T>>>>,
}

impl<'a, T: 'static + std::marker::Send> Tasks<'a, T> {
    fn spawn<F: 'static>(&self, exec_task: F, thread_num: u8)
    where
        F: Fn(&Arc<Mutex<Vec<TaskSpawner<T>>>>, u8) + std::marker::Send + Copy,
    {
        let mut thread_handles = Vec::new();
        for i in 0..thread_num {
            let cloned_arc_data = self.get_arc_clone();
            let handle = thread::spawn(move || {
                exec_task(&cloned_arc_data, i);
            });
            thread_handles.push(handle);
        }
        for handle in thread_handles {
            handle.join().unwrap();
        }
    }
    fn get_arc_clone(&self) -> Arc<Mutex<Vec<TaskSpawner<T>>>> {
        Arc::clone(self.arc_data)
    }
}

/// Spawn Tasks
pub fn spawn_threads<F: 'static>(exec_task: F, to_spawn: u8) -> Arc<Mutex<Vec<TaskSpawner<String>>>>
where
    F: Fn(&Arc<Mutex<Vec<TaskSpawner<String>>>>, u8) + std::marker::Send + Copy,
{
    let arc_data: Arc<Mutex<Vec<TaskSpawner<String>>>> = Arc::new(Mutex::new(Vec::new()));
    let task_spawner = Tasks {
        arc_data: &arc_data,
    };
    task_spawner.spawn(exec_task, to_spawn);
    arc_data
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_multi_thread() {
        use super::*;
        let to_spawn = 20;
        let exec_task = |arc_data: &Arc<Mutex<Vec<TaskSpawner<String>>>>, id: u8| {
            // do some work
            // ...
            // ...
            // ...
            // save result for later use
            let mut spawned_task = TaskSpawner::new(id as u32);
            spawned_task.set_task_data("Thread finished".to_owned());
            spawned_task.set_success(true);
            let mut arc_data = arc_data.lock().unwrap();
            arc_data.push(spawned_task);
        };
        let arc_data = spawn_threads(exec_task, to_spawn);
        arc_data.lock().unwrap().iter().for_each(|x| {
            println!("({}) - {} -> {:#?}", x.id, x.data, x.success);
        });
        assert_eq!(arc_data.lock().unwrap().len() as u8, to_spawn);
    }
}
