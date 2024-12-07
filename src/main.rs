use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{Read, Write};

const FILE_NAME: &str = "tasks.txt";

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct Task {
    id: usize,
    description: String,
    completed: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct TaskRepository {
    tasks: Vec<Task>,
    next_id: usize,
}

impl TaskRepository {
    fn add_task(&mut self, description: String) {
        self.tasks.push(Task {
            id: self.next_id,
            description,
            completed: false,
        });
        self.next_id += 1;
    }

    fn edit_task(&mut self, id: usize, description: String) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.description = description;
        }
    }

    fn delete_task(&mut self, id: usize) {
        self.tasks.retain(|task| task.id != id);
    }

    fn mark_completed(&mut self, id: usize) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.completed = true;
        }
    }

    fn save_to_file(&self) {
        if let Ok(data) = ron::to_string(self) {
            if let Ok(mut file) = File::create(FILE_NAME) {
                file.write_all(data.as_bytes());
            }
        }
    }

    fn load_from_file() -> Self {
        if let Ok(mut file) = File::open(FILE_NAME) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(manager) = ron::from_str(&contents) {
                    return manager;
                }
            }
        }
        TaskRepository::default()
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "todo app",
        options,
        Box::new(|_cc| Ok(Box::new(ToDoApp::new()))),
    );
}

struct ToDoApp {
    manager: TaskRepository,
    new_description: String,
    edit_description: String,
    show_edit_popup: bool,
    edit_id_task: Option<usize>,
}

impl ToDoApp {
    fn new() -> Self {
        Self {
            manager: TaskRepository::load_from_file(),
            new_description: String::new(),
            edit_description: String::new(),
            show_edit_popup: false,
            edit_id_task: None
        }
    }
}

impl eframe::App for ToDoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("To-Do List");

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_description);
                if ui.button("Add Task").clicked() && !self.new_description.is_empty() {
                    self.manager.add_task(self.new_description.clone());
                    self.new_description.clear();
                    self.manager.save_to_file();
                }
            });

            ui.separator();

            //&self.manager.tasks immutable -> cloning tasks
            let tasks = self.manager.tasks.clone();
            // display all tasks
            for task in tasks {
                ui.horizontal(|ui| {
                    //make strikethrough text is task is completed
                    if task.completed {
                        ui.label(egui::RichText::new(&task.description).strikethrough());
                    } else {
                        ui.label(&task.description);
                    }

                    if ui.button("Edit").clicked() {
                        self.show_edit_popup = true;
                        self.edit_id_task = Some(task.id);
                        self.edit_description = task.description.clone();
                    }

                    if !task.completed {
                        if ui.button("Complete").clicked() {
                            self.manager.mark_completed(task.id);
                            self.manager.save_to_file();
                        }
                    }

                    if ui.button("Delete").clicked() {
                        self.manager.delete_task(task.id);
                        self.manager.save_to_file();
                    }
                });
            }
            //window for editing task
            if self.show_edit_popup {
                egui::Window::new("Edit Task Description")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label("Enter new description:");
                        ui.text_edit_singleline(&mut self.edit_description);

                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                if let Some(task_id) = self.edit_id_task {
                                    self.manager.edit_task(task_id, self.edit_description.clone());
                                    self.manager.save_to_file();
                                }
                                self.show_edit_popup = false;
                            }

                            if ui.button("Cancel").clicked() {
                                self.show_edit_popup = false;
                            }
                        });
                    });
            }

            if self.manager.tasks.is_empty() {
                ui.label("No tasks available.");
            }
        });
    }
}

