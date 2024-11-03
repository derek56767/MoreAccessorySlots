#![feature(lazy_cell, ptr_sub_ptr)]
use std::cmp::Ordering;

use unity::prelude::*;
use unity::engine::Sprite;
use unity::engine::FilterMode; 
use unity::engine::Texture2D;
use unity::engine::ImageConversion;
use unity::engine::Rect;
use unity::engine::Vector2;
use unity::engine::SpriteMeshType;

use engage::{
    stream::Stream,
    gameicon::GameIcon,
    gamedata::{
        accessory::AccessoryData, unit::{
            UnitAccessory,
            UnitAccessoryList
        }, Gamedata
    },
};

use skyline::{
    hook,
    hooks::InlineCtx,
};

use include_dir::{include_dir, Dir};

static ICON_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources");

#[macro_export]
macro_rules! reg_x {
    ($ctx:ident, $no:expr) => {
        unsafe { *$ctx.registers[$no].x.as_ref() }
    };
}

// Due to how you use the enum, let's not use these for now
#[repr(i32)]
pub enum AccessoryDataMasks {
    Body = 1,
    Head = 2,
    Face = 4,
    Back = 8,
    // Expand here
}

// Due to how you use the enum, let's not use these for now
#[repr(i32)]
pub enum AccessoryDataKinds {
    Body = 0,
    Head = 1,
    Face = 2,
    Back = 3,
}

//Need to edit this for new accessory types to appear in the store.
#[unity::hook("App", "UnitAccessoryList", "get_Count")]
pub fn unitaccessorylist_get_count(_this: &mut UnitAccessoryList, _method_info: OptionalMethod) -> i32 {
    return 16;
}

#[unity::hook("App", "AccessoryData", "OnBuild")]
pub fn accessorydata_on_build_hook(this: &mut AccessoryData, method_info: OptionalMethod) {
    //Takes the Mask value from the accessory's XML data and assigns it a Kind.
    call_original!(this, method_info);

    if this.mask > 8
    {
        match this.mask{
            16 => this.kind = 4,
            32 => this.kind = 5,
            64 => this.kind = 6,
            128 => this.kind = 7,
            256 => this.kind = 8,
            512 => this.kind = 9,
            1024 => this.kind = 10,
            2048 => this.kind = 11,
            4096 => this.kind = 12,
            8192 => this.kind = 13,
            16384 => this.kind = 14,
            32768 => this.kind = 15,
            65536 => this.kind = 16,
            _=> this.kind = 1,
        }
    }
}

#[unity::hook("App", "UnitAccessoryList", "CopyFrom")]
pub fn unitaccessorylist_copyfrom_hook(this: &mut UnitAccessoryList, list: &mut UnitAccessoryList, _method_info: OptionalMethod) {
    //Copies the contents of one accessory list to another.
    this.unit_accessory_array
        .iter_mut()
        .zip(list.unit_accessory_array.iter_mut())
        .for_each(|(dest, src)| {
            dest.index = src.index;
        });
}

#[unity::hook("App", "UnitAccessoryList", "Clear")]
pub fn unitaccessorylist_clear_hook(this: &mut UnitAccessoryList, _method_info: OptionalMethod) {
    //Removes every accessory in an Accessory List.
    this.unit_accessory_array.iter_mut().for_each(|acc| acc.index = 0);
}

#[skyline::hook(offset = 0x1f620a0)]
pub fn unitaccessorylist_add_hook(this: &mut UnitAccessoryList, accessory: Option<&mut AccessoryData>, index: usize, _method_info: OptionalMethod) -> bool {
    // Index is always, ALWAYS, the accessory's Kind, which is calculated based on the "Mask" value in it's XML Data during the "OnBuild" function.
    // Ray: I have some clue why this is done but I hope you do too because I'll forget.

    let accessories = AccessoryData::get_list().expect("Couldn't reach AccessoryData List");

    if let Some(accessory) = accessory {
        for curr_acc in this.unit_accessory_array.iter_mut() { // Go through every entry in the array.
            // Grab the AccessoryData at that index in the XML
            if let Some(found) = accessories.get(curr_acc.index as usize) {
                // If an entry was found, check if the mask is similar and set the index to 0 if it is, to unequip other accessories that share the slot.
                if accessory.mask == found.mask {
                    curr_acc.index = 0;
                }
            }
        }

        // this.unit_accessory_array
        //     .get_mut(index)
        //     .map(|entry| entry.index = accessory.parent.index)
        //     .expect("AccessoryKind goes beyond the expected range");

        if index > this.unit_accessory_array.len() {
            for item in this.unit_accessory_array.iter_mut() {
                if item.index == 0 {
                    item.index = accessory.parent.index;
                }
            }
        } else {
            this.unit_accessory_array[index].index = accessory.parent.index;
        }

        true
    } else {
        false
    }
}

#[unity::hook("App", "UnitAccessoryList", "IsExist")]
pub fn unitaccessorylist_is_exist_hook(this: &mut UnitAccessoryList, accessory: Option<&mut AccessoryData>, _method_info: OptionalMethod) -> bool {
    let accessories = AccessoryData::get_list().expect("Couldn't reach AccessoryData List");

    // This is your old "if accessory == 0x0 {}". In the context of talking with C, Rust allows you to use Option<> on a pointer to signify that it could be null.
    // That gives you plenty of fancy ways to check for null
    accessory.is_some_and(|accessory| {
        // Looks for the AID of the provided accessory in the XML and return the index of the matching entry
        this.unit_accessory_array
            .iter() // Go through every entry in the array.
            .any(|curr_acc| { // Confirms if any of the items in the array fulfills the condition.
                // Grab the AccessoryData at that index in the XML if it's present, and if it is, compare the AIDs.
                // Return false if the index is out of bounds OR the AIDs don't match
                accessories.get(curr_acc.index as usize).is_some_and(|item| {
                    item.aid.get_string().unwrap() == accessory.aid.get_string().unwrap()
                })
            })
    })
}

#[unity::hook("App", "UnitAccessoryList", "Serialize")]
pub fn unitaccessorylist_serialize_hook(this: &mut UnitAccessoryList, stream: &mut Stream, _method_info: OptionalMethod) {
    stream.write_int(1).expect("Could not write version number when serializing UnitAccessoryList");

    // TODO: Simplify by calling serialize on the UnitAccessoryList directly
    this.unit_accessory_array
        .iter_mut()
        .for_each(|curr_acc| {
            curr_acc.serialize(stream);
        });
}

#[unity::hook("App", "UnitAccessoryList", "Deserialize")]
pub fn unitaccessorylist_deserialize_hook(this: &mut UnitAccessoryList, stream: &mut Stream, _method_info: OptionalMethod) {
    this.unit_accessory_array
            .iter_mut()
            .for_each(|curr_acc| {
                curr_acc.index = 0;
            });

    let version_check = stream.read_int().expect("Could not read the version from the UnitAccessoryList block in the savefile");

    if version_check > 0 {
        // Deserializes as many items as there are in the array
        this.unit_accessory_array.iter_mut()
            .for_each(|curr_acc| {
                curr_acc.deserialize(stream);
            });
    } else {
        // Just deserializes the 4 original items
        this.unit_accessory_array[..4].iter_mut()
            .for_each(|curr_acc| {
                curr_acc.deserialize(stream);
            });
        // Unequips all accessories this first load because apparently accessories 
        // can get stuck equipped due to the Kinds being changed since the save was made.
        this.unit_accessory_array[..4].iter_mut()
            .for_each(|curr_acc| {
                curr_acc.index = 0;
            });
    }
}

#[unity::hook("App", "GameIcon", "TryGetAccessoryKinds")]
pub fn gameicon_try_get_accessory_kinds_hook(accessory_kinds: i32, _method_info: OptionalMethod) -> &'static Sprite
{
    //Do not use Kind 4.  The game reserves that one for Sommie accessories
    let i = match accessory_kinds {
        0 => return GameIcon::try_get_system("Clothes").expect("Couldn't get sprite for AccessoryKind"),
        1 => "Hat.png",
        
        2 => return GameIcon::try_get_system("Face").expect("Couldn't get sprite for AccessoryKind"),
        3 => "Pose.png",
        5 => "BattleOutfit.png",
        6 => "Dye.png",
        7 => "Style.png",
        8 => "Head.png",
        9 => "Hair.png",
        10 => "Scaling.png",
        11 => "SkinColor.png",
        12 => "Voice.png",
        13 => "MaskColor.png",
        14 => "Accessory2.png",
        15 => "NoEngage.png",
        16 => "Placeholder.png",
        _=> "Placeholder.png",
    };

    // Confirm this code actually works properly at some point.
    let texture_png = ICON_DIR.get_file(i)
        .unwrap_or_else(|| {
            panic!("Expanded Accessory Slot plugin could not find icon with name '{}'. Consider adding it to the 'resources' directory.", i);
        })
        .contents()
        .to_owned(); // Necessary because Il2CppArray currently swaps the slice with its content

    let array = Il2CppArray::from_slice(texture_png).unwrap();
    
    let new_texture = Texture2D::new(48, 48);
    
    if !ImageConversion::load_image(new_texture, array) {
        panic!("Could not load the job icon for '{}'.\n\nMake sure it is a PNG file with a dimension of 48x48 pixels", i);
    }
    
    new_texture.set_filter_mode(FilterMode::Trilinear);
    
    let rect = Rect::new(0.0, 0.0, 48.0, 48.0);
    let pivot = Vector2::new(0.5, 0.5);
    
    let sprite = Sprite::create2(new_texture, rect, pivot, 100.0, 1, SpriteMeshType::Tight);
    return sprite;   
}

#[skyline::hook(offset = 0x27b5eb0, inline)]
fn accessorydetail_hook(ctx: &mut InlineCtx) {
    let kind = reg_x!(ctx, 22) as usize;
    let mut out_mid = reg_x!(ctx, 8) as *mut*const Il2CppString;

    let mid = match kind {
        5 => "MID_MENU_ACCESSORY_SHOP_PART_OUTFIT_BATTLE",
        6 => "MID_MENU_ACCESSORY_SHOP_PART_DYE",
        7 => "MID_MENU_ACCESSORY_SHOP_PART_STYLE",
        8 => "MID_MENU_ACCESSORY_SHOP_PART_HEAD",
        9 => "MID_MENU_ACCESSORY_SHOP_PART_HAIR",
        10 => "MID_MENU_ACCESSORY_SHOP_PART_SCALING",
        11 => "MID_MENU_ACCESSORY_SHOP_PART_SKINCOLOR",
        12 => "MID_MENU_ACCESSORY_SHOP_PART_VOICE",
        13 => "MID_MENU_ACCESSORY_SHOP_PART_MASKCOLOR",
        14 => "MID_MENU_ACCESSORY_SHOP_PART_ACCESSORY2",
        15 => "MID_MENU_ACCESSORY_SHOP_PART_NOENGAGE",
        16 => "MID_MENU_ACCESSORY_SHOP_PART_PLACEHOLDER",
        _=> "MID_MENU_ACCESSORY_SHOP_PART_PLACEHOLDER",
    };

    unsafe{*out_mid = Il2CppString::new(mid) as *const _; }

}

#[unity::hook("App", "UnitAccessoryList", ".ctor")]
pub fn unitaccessorylist_ctor_hook(this: &mut UnitAccessoryList, method_info: OptionalMethod,)
{
    call_original!(this, method_info);

    // Il2CppArray can be turned into a slice (https://doc.rust-lang.org/std/primitive.slice.html) and slices can be iterated (https://doc.rust-lang.org/std/iter/trait.Iterator.html) on, so we can just walk through every item in the array and manipulate them
    // println!("Array length: {}", this.unit_accessory_array.len());

    this.unit_accessory_array
        .iter_mut()
        .for_each(|item| {
            *item = UnitAccessory::instantiate()
                .map(|acc| {
                    acc.index = 0 as i32;
                    acc
                })
                .unwrap();
        });
}

#[skyline::main(name = "TestProject")]
pub fn main() {
    // Install a panic handler for your plugin, allowing you to customize what to do if there's an issue in your code.
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        // Some magic thing to turn what was provided to the panic into a string. Don't mind it too much.
        // The message will be stored in the msg variable for you to use.
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };

        // This creates a new String with a message of your choice, writing the location of the panic and its message inside of it.
        // Note the \0 at the end. This is needed because show_error is a C function and expects a C string.
        // This is actually just a result of bad old code and shouldn't be necessary most of the time.
        let err_msg = format!(
            "Extended Accessory Slots has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );

        // We call the native Error dialog of the Nintendo Switch with this convenient method.
        // The error code is set to 69 because we do need a value, while the first message displays in the popup and the second shows up when pressing Details.
        skyline::error::show_error(
            69,
            "Custom plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
    
    skyline::patching::Patch::in_text(0x027b5eb0).bytes(&[0x1F, 0x20, 0x03, 0xD5]).expect("Couldn’t patch that shit for some reasons");

    skyline::install_hooks!(
        accessorydata_on_build_hook,
        gameicon_try_get_accessory_kinds_hook,
        unitaccessorylist_ctor_hook,
        unitaccessorylist_serialize_hook,
        unitaccessorylist_deserialize_hook,
        unitaccessorylist_copyfrom_hook,
        unitaccessorylist_get_count,
        unitaccessorylist_clear_hook,
        unitaccessorylist_is_exist_hook,
        unitaccessorylist_add_hook,
        accessorydetail_hook
    );

    //Patches the length of UnitAccessoryList in it's ctor function.
    skyline::patching::Patch::in_text(0x01f61c00).bytes(&[0x01, 0x02, 0x80, 0x52]).expect("Couldn’t patch that shit for some reasons");
    
    skyline::patching::Patch::in_text(0x027b5d70).bytes(&[0xDF, 0x3E, 0x00, 0x71]).expect("Couldn’t patch that shit for some reasons");
    skyline::patching::Patch::in_text(0x027b5d8c).bytes(&[0xDF, 0x42, 0x00, 0x71]).expect("Couldn’t patch that shit for some reasons");


    

    //skyline::patching::Patch::in_text(0x027b5e7c).bytes(&[0xDF, 0x02, 0x00, 0x71]).expect("Couldn’t patch that shit for some reasons");
    //skyline::patching::Patch::in_text(0x027b5e80).bytes(&[0x21, 0x01, 0x00, 0x54]).expect("Couldn’t patch that shit for some reasons");

    //Patch Get_Count
    //skyline::patching::Patch::in_text(0x01f61b10).bytes(&[0xE0, 0x01, 0x80, 0x52]).expect("Couldn’t patch that shit for some reasons");

    //skyline::patching::Patch::in_text(0x027bffcc).bytes(&[0x1F, 0x20, 0x03, 0xD5]).expect("Couldn’t patch that shit for some reasons");
}