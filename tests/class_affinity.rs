use fate_grand_calculator::model::ClassType;

#[test]
fn class_affinity_uses_attacker_to_enemy_direction() {
    assert_eq!(ClassType::Saber.affinity_against(ClassType::Lancer), 2.0);
    assert_eq!(ClassType::Lancer.affinity_against(ClassType::Saber), 0.5);
    assert_eq!(ClassType::Ruler.affinity_against(ClassType::Saber), 1.0);
    assert_eq!(ClassType::Saber.affinity_against(ClassType::Ruler), 0.5);
    assert_eq!(ClassType::Pretender.affinity_against(ClassType::Saber), 1.5);
}
