struct Servant {
    //servant stats
    name: String,
    level: u32,
    max_hp: u32,
    attack: u32,
    card: CardType,
    class: ClassType,
    attribute: AttributeType,
    noble_phantasm: NoblePhantasmType,
    skill_1: SkillType,
    skill_2: SkillType,
    skill_3: SkillType,

}

impl Servant {
    fn new(
        name: String,
        level: u32,
        max_hp: u32,
        attack: u32,
        card: CardType,
        class: ClassType,
        attribute: AttributeType,
        noble_phantasm: NoblePhantasmType,
    ) -> Self {
        Servant {
            name,
            level,
            max_hp,
            attack,
            card,
            class,
            attribute,
            noble_phantasm,
        }
    }
}