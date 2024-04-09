use gloo::storage::{LocalStorage, Storage};
use state::{Entry, Filter, State};
use strum::IntoEnumIterator;
use web_sys::HtmlInputElement;
use yew::{html::Scope, Component, Context, KeyboardEvent, NodeRef, TargetCast};
use yew::{classes, html, Classes, FocusEvent, Html};
mod state;
mod db;

const KEY: &str = "yew.todomvc.self";

pub struct App {
    state: State,
    focus_ref: NodeRef,
}

pub enum Msg {
    Add(String),
    ToggleAll,
    ToggleEdit(usize),
    Toggle(usize),
    Remove(usize),
    Edit(usize, String),
    Focus,
    Filter(Filter),
    ClearCompleted,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let entries = LocalStorage::get(KEY).unwrap_or_else(|_| vec![]);
        let state = State {
            entries,
            filter: Filter::All,
            edit_value: String::new(),
        };
        let focus_ref = NodeRef::default();
        Self { state, focus_ref }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Add(description) => {
                let description = description.trim();
                if !description.is_empty() {
                    let entry = Entry {
                        description: description.to_string(),
                        completed: false,
                        editing: false,
                    };
                    self.state.entries.push(entry);
                }
            }
            Msg::ToggleAll => {
                let status = self.state.is_all_completed();
                self.state.toggle_all(status);
            }
            Msg::Toggle(idx) => {
                self.state.toggle(idx);
            }
            Msg::ToggleEdit(idx) => {
                self.state.toggle_edit(idx);
            }
            Msg::Remove(idx) => {
                self.state.remove(idx);
            }
            Msg::Edit(idx, val) => {
                self.state.complete_edit(idx, val.trim().to_string());
                self.state.edit_value = String::new();
            }
            Msg::Focus => {
                if let Some(input) = self.focus_ref.cast::<HtmlInputElement>() {
                    input.focus().expect("focus input");
                }
            }
            Msg::Filter(filter) => {
                self.state.filter = filter;
            }
            Msg::ClearCompleted => {
                self.state.clear_completed();
            }
        }
        LocalStorage::set(KEY, &self.state.entries).expect("failed to set entries in storage");
        true
    }
    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        let hidden_class = if self.state.entries.is_empty() { "hidden" } else { "" };
        html! {
            <div class="todo_wrapper">
                <section class="todoapp">
                    <header class="header">
                        <h1>{"todos"}</h1>
                        { self.view_input(ctx.link()) }
                    </header>
                    <section class={classes!("main", hidden_class)}>
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            checked={self.state.is_all_completed()}
                            onclick={ctx.link().callback(|_| Msg::ToggleAll)}
                        />
                        <label for="toggle-all" />
                        <ul class="todo-list">
                            { for self.state.entries.iter()
                                .filter(|e| self.state.filter.fits(e))
                                .enumerate()
                                .map(|e| self.view_entry(e, ctx.link()))
                            }
                        </ul>
                    </section>
                    <footer class={classes!("footer", hidden_class)}>
                        <span class="todo-count">
                            <strong>
                                { self.state.total() }
                            </strong>
                            { " item(s) left" }
                        </span>
                        <ul class="filters">
                            { for Filter::iter().map(|f| self.view_filter(f, ctx.link())) }
                        </ul>
                        <button
                            class="clear-completed"
                            onclick={ctx.link().callback(|_| Msg::ClearCompleted)}
                        >
                            { format!("Clear completed ({})", self.state.completed_count())}
                        </button>
                    </footer>
                </section>
                <footer class="info">
                    <p>{"Double-click to edit a todo"}</p>
                    <p>{"Written by Li Hang"}</p>
                </footer>
            </div>
        }
    }
}

impl App {
    fn view_input(&self, link: &Scope<Self>) -> yew::Html {
        let onkeypress = link.batch_callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: HtmlInputElement = e.target_unchecked_into();
                let value = input.value();
                input.set_value("");
                Some(Msg::Add(value))    
            } else {
                None
            }
        });
        html! {
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                { onkeypress }
            />
        }
    }
    fn view_entry(&self, (idx, entry): (usize, &Entry), link: &Scope<Self>) -> Html {
        let mut class = Classes::from("todo");
        if entry.editing {
            class.push("editing");
        }
        if entry.completed {
            class.push("completed");
        }
        html! {
            <li {class}>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked={entry.completed}
                        onclick={link.callback(move |_| Msg::Toggle(idx))}
                        />
                    <label ondblclick={link.callback(move |_| Msg::ToggleEdit(idx))}>
                        {&entry.description}
                    </label>
                    <button
                        class="destroy"
                        onclick={link.callback(move |_| Msg::Remove(idx))}
                    />
                </div>
                { self.view_entry_edit_input((idx, entry), link) }
            </li>
        }
    }

    fn view_entry_edit_input(&self, (idx, entry): (usize, &Entry), link: &Scope<Self>) -> Html {
        let edit = move |input: HtmlInputElement| {
            let value = input.value();
            input.set_value("");
            Msg::Edit(idx, value)
        };

        let onblur = link.callback(move |e: FocusEvent| edit(e.target_unchecked_into()));

        let onkeypress = link.batch_callback(move |e: KeyboardEvent| {
            (e.key() == "Enter").then(|| edit(e.target_unchecked_into()))
        });

        if entry.editing {
            html! {
                <input
                    class="edit"
                    ref={self.focus_ref.clone()}
                    value={self.state.edit_value.clone()}
                    onmouseover={link.callback(|_| Msg::Focus)}
                    onblur={onblur}
                    onkeypress={onkeypress}
                />
            }
        } else {
            html! { <input type="hidden" /> }
        }
    }
    fn view_filter(&self, filter: Filter, link: &Scope<Self>) -> Html {
        let active = if self.state.filter == filter {
            "selected"
        } else {
            "not-selected"
        };
        html! {
            <li>
                <a
                    class={active}
                    href={filter.as_href()} // 暂无效果
                    onclick={link.callback(move |_| Msg::Filter(filter))}
                >
                    { filter }
                </a>
            </li>
        }
    }
}