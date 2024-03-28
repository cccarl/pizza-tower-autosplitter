#[derive(PartialEq)]
pub enum Level {
    Hub,
    F1Tutorial,
    F1TutorialNoise,
    F1JohnGutter,
    F1Pizzascape,
    F1AncientCheese,
    F1BloodsauceDungeon,
    F2OreganoDesert,
    F2Wasteyard,
    F2FunFarm,
    F2FastfoodSaloon,
    F3CrustCove,
    F3GnomeForest,
    F3Golf,
    F3DeepDish9,
    F4ThePigCity,
    F4OhShit,
    F4PeppibotFactory,
    F4Refrigerator,
    F5Pizzascare,
    F5DMAS,
    F5War,
    F5CrumblingTower,
    SecretsOfTheWorld,
    TrickyTreat,
    Pepperman,
    Vigilante,
    Noise,
    Fake,
    PizzaFace,
    ResultsScreen,
    Unknown,
}

pub fn get_current_level(room_name: &str, prev_level: Level) -> Level {
    // special cases for rooms that overlap in multiple levels
    if prev_level == Level::F5CrumblingTower
        && room_name.contains("tower_")
        && room_name != "tower_pizzafacehall"
    {
        return Level::F5CrumblingTower;
    }

    if prev_level == Level::SecretsOfTheWorld && room_name.contains("secret") {
        return Level::SecretsOfTheWorld;
    }

    match room_name {
        "tower_finalhallway" => Level::F5CrumblingTower,
        x if x.contains("tower_tutorial1N")
            || x.contains("tower_tutorial2N")
            || x.contains("tower_tutorial3N") =>
        {
            Level::F1TutorialNoise
        }
        x if x.contains("tower_tutorial") => Level::F1Tutorial,
        x if x.contains("tower_") || x == "boss_pizzafacehub" => Level::Hub,
        x if x.contains("entrance_") => Level::F1JohnGutter,
        x if x.contains("medieval_") => Level::F1Pizzascape,
        x if x.contains("ruin_") => Level::F1AncientCheese,
        x if x.contains("dungeon_") => Level::F1BloodsauceDungeon,
        "boss_pepperman" => Level::Pepperman,
        x if x.contains("badland_") => Level::F2OreganoDesert,
        x if x.contains("graveyard_") => Level::F2Wasteyard,
        x if x.contains("farm_") => Level::F2FunFarm,
        x if x.contains("saloon_") => Level::F2FastfoodSaloon,
        "boss_vigilante" => Level::Vigilante,
        x if x.contains("plage_") => Level::F3CrustCove,
        x if x.contains("forest_") => Level::F3GnomeForest,
        x if x.contains("space_") => Level::F3DeepDish9,
        x if x.contains("minigolf_") => Level::F3Golf,
        "boss_noise" => Level::Noise,
        x if x.contains("street_") => Level::F4ThePigCity,
        x if x.contains("industrial_") => Level::F4PeppibotFactory,
        x if x.contains("sewer_") => Level::F4OhShit,
        x if x.contains("freezer_") => Level::F4Refrigerator,
        x if x.contains("boss_fakepep") => Level::Fake,
        x if x.contains("secret_entrance") => Level::SecretsOfTheWorld,
        x if x.contains("trickytreat") => Level::TrickyTreat,
        x if x.contains("chateau_") => Level::F5Pizzascare,
        x if x.contains("kidsparty_") => Level::F5DMAS,
        x if x.contains("war_") => Level::F5War,
        "boss_pizzaface" => Level::PizzaFace,
        "rank_room" => Level::ResultsScreen,
        _ => Level::Unknown, // where did you go?
    }
}

pub fn get_starting_room<'a>(level: &Level) -> &'a str {
    match level {
        Level::F1Tutorial => "tower_tutorial1",
        Level::F1TutorialNoise => "tower_tutorial1N",
        Level::F1JohnGutter => "entrance_1",
        Level::F1Pizzascape => "medieval_1",
        Level::F1AncientCheese => "ruin_1",
        Level::F1BloodsauceDungeon => "dungeon_1",
        Level::F2OreganoDesert => "badland_1",
        Level::F2Wasteyard => "graveyard_1",
        Level::F2FunFarm => "farm_2",
        Level::F2FastfoodSaloon => "saloon_1",
        Level::F3CrustCove => "plage_entrance",
        Level::F3GnomeForest => "forest_1",
        Level::F3Golf => "minigolf_1",
        Level::F3DeepDish9 => "space_1",
        Level::F4ThePigCity => "street_intro",
        Level::F4OhShit => "sewer_1",
        Level::F4PeppibotFactory => "industrial_1",
        Level::F4Refrigerator => "freezer_1",
        Level::F5Pizzascare => "chateau_1",
        Level::F5DMAS => "kidsparty_1",
        Level::F5War => "war_1",
        Level::F5CrumblingTower => "tower_finalhallway",
        Level::SecretsOfTheWorld => "secret_entrance",
        Level::TrickyTreat => "trickytreat_2",
        Level::Pepperman => "boss_pepperman",
        Level::Vigilante => "boss_vigilante",
        Level::Noise => "boss_noise",
        Level::Fake => "boss_fakepep",
        Level::PizzaFace => "boss_pizzaface",
        _ => "-",
    }
}

/**
 * Returns true if a key room that should enable the split for the current level (in full game) is received
 */
pub fn full_game_split_unlock_rooms(current_room: &str) -> bool {
    [
        "tower_tutorial10",
        "tower_tutorial3N",
        "entrance_10",
        "medieval_10",
        "ruin_11",
        "dungeon_10",
        "badland_9",
        "graveyard_6",
        "farm_11",
        "saloon_6",
        "plage_cavern2",
        "forest_john",
        "space_9",
        "minigolf_8",
        "street_john",
        "sewer_8",
        "industrial_5",
        "freezer_escape1",
        "chateau_9",
        "kidsparty_john",
        "war_1",
        "boss_pepperman",
        "boss_vigilante",
        "boss_noise",
        "boss_fakepepkey",
        "boss_pizzaface",
        "tower_finalhallway",
    ]
    .contains(&current_room)
}

/**
 * Return true if it receives a room that should trigger a split, usually where the levels end
 */
pub fn full_game_split_rooms(exited_level: &str) -> bool {
    [
        "tower_tutorial1",
        "tower_tutorial1N",
        "entrance_1",
        "medieval_1",
        "ruin_1",
        "dungeon_1",
        "badland_1",
        "graveyard_1",
        "farm_2",
        "saloon_1",
        "plage_entrance",
        "forest_1",
        "minigolf_1",
        "space_1",
        "street_intro",
        "sewer_1",
        "industrial_1",
        "freezer_1",
        "chateau_1",
        "kidsparty_1",
        "war_13",
        "boss_pepperman",
        "boss_vigilante",
        "boss_noise",
        "boss_fakepepkey",
        "boss_pizzaface",
        "boss_pizzafacefinale",
        "tower_entrancehall",
        "rank_room",
    ]
    .contains(&exited_level)
}

pub fn get_full_level_name<'a>(level: &Level) -> &'a str {
    match level {
        Level::Hub => "Hub",
        Level::F1Tutorial => "Tutorial",
        Level::F1TutorialNoise => "Tutorial (Noise)",
        Level::F1JohnGutter => "John Gutter",
        Level::F1Pizzascape => "Pizzascape",
        Level::F1AncientCheese => "Ancient Cheese",
        Level::F1BloodsauceDungeon => "Bloodsauce Dungeon",
        Level::F2OreganoDesert => "Oregano Desert",
        Level::F2Wasteyard => "Wasteyard",
        Level::F2FunFarm => "Fun Farm",
        Level::F2FastfoodSaloon => "Fastfood Saloon",
        Level::F3CrustCove => "Crust Cove",
        Level::F3GnomeForest => "Gnome Forest",
        Level::F3Golf => "GOLF",
        Level::F3DeepDish9 => "Deep-Dish 9",
        Level::F4ThePigCity => "The Pig City",
        Level::F4OhShit => "Oh Shit!",
        Level::F4PeppibotFactory => "Peppibot Factory",
        Level::F4Refrigerator => "Refrigerator-Refrigerador-Freezerator",
        Level::F5Pizzascare => "Pizzascare",
        Level::F5DMAS => "Don't Make a Sound",
        Level::F5War => "WAR",
        Level::F5CrumblingTower => "The Crumbling Tower of Pizza",
        Level::SecretsOfTheWorld => "Secrets of the World",
        Level::TrickyTreat => "Tricky Treat",
        Level::Pepperman => "Pepperman",
        Level::Vigilante => "The Vigilante",
        Level::Noise => "The Noise/The Doise",
        Level::Fake => "Fake Peppino",
        Level::PizzaFace => "Pizzaface",
        Level::ResultsScreen => "You did it! Pizza Tower",
        Level::Unknown => "idk",
    }
}
