use std::collections::HashMap;

use muda::{
    accelerator::Accelerator, CheckMenuItem, IsMenuItem, Menu, MenuId, MenuItem,
    PredefinedMenuItem, Submenu,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum MenuItemDef {
    #[serde(rename = "normal")]
    Normal {
        id: String,
        label: String,
        accelerator: Option<String>,
        enabled: Option<bool>,
    },
    #[serde(rename = "check")]
    Check {
        id: String,
        label: String,
        checked: Option<bool>,
        enabled: Option<bool>,
    },
    #[serde(rename = "separator")]
    Separator,
    #[serde(rename = "submenu")]
    Submenu {
        label: String,
        enabled: Option<bool>,
        items: Vec<MenuItemDef>,
    },
}

pub struct BuiltMenu {
    pub menu: Menu,
    pub id_map: HashMap<MenuId, String>,
}

pub fn build_menu(json: &str) -> Option<BuiltMenu> {
    let defs: Vec<MenuItemDef> = serde_json::from_str(json).ok()?;
    let menu = Menu::new();
    let mut id_map = HashMap::new();
    for def in &defs {
        if let Some(item) = build_item(def, &mut id_map) {
            let _ = menu.append(item.as_ref());
        }
    }
    Some(BuiltMenu { menu, id_map })
}

fn build_item(
    def: &MenuItemDef,
    id_map: &mut HashMap<MenuId, String>,
) -> Option<Box<dyn IsMenuItem>> {
    match def {
        MenuItemDef::Normal {
            id,
            label,
            accelerator,
            enabled,
        } => {
            let accel = accelerator
                .as_deref()
                .and_then(|a| a.parse::<Accelerator>().ok());
            let item = MenuItem::with_id(MenuId::new(id), label, enabled.unwrap_or(true), accel);
            id_map.insert(item.id().clone(), id.clone());
            Some(Box::new(item))
        }
        MenuItemDef::Check {
            id,
            label,
            checked,
            enabled,
        } => {
            let item = CheckMenuItem::with_id(
                MenuId::new(id),
                label,
                enabled.unwrap_or(true),
                checked.unwrap_or(false),
                None::<Accelerator>,
            );
            id_map.insert(item.id().clone(), id.clone());
            Some(Box::new(item))
        }
        MenuItemDef::Separator => Some(Box::new(PredefinedMenuItem::separator())),
        MenuItemDef::Submenu {
            label,
            enabled,
            items,
        } => {
            let submenu = Submenu::new(label, enabled.unwrap_or(true));
            for child in items {
                if let Some(item) = build_item(child, id_map) {
                    let _ = submenu.append(item.as_ref());
                }
            }
            Some(Box::new(submenu))
        }
    }
}
