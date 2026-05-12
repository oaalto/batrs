//! Command lines for casting **floating disc** and moving items on **my disc**
//! (patterns for magical and civil spell output).

use super::client_send_line;

/// `@cast floating disc`
pub fn send_cast_floating_disc() -> String {
    client_send_line("cast floating disc")
}

/// `@get all from my disc`
pub fn send_get_all_from_disc() -> String {
    client_send_line("get all from my disc")
}

/// `@get all armour from my disc`
pub fn send_get_all_armour_from_disc() -> String {
    client_send_line("get all armour from my disc")
}

/// `@get all weapon from my disc`
pub fn send_get_all_weapon_from_disc() -> String {
    client_send_line("get all weapon from my disc")
}

/// `@put noeq in my disc`
pub fn send_put_noeq_in_disc() -> String {
    client_send_line("put noeq in my disc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_match_expected_aliases() {
        assert_eq!(send_cast_floating_disc(), "@cast floating disc");
        assert_eq!(send_get_all_from_disc(), "@get all from my disc");
        assert_eq!(
            send_get_all_armour_from_disc(),
            "@get all armour from my disc"
        );
        assert_eq!(
            send_get_all_weapon_from_disc(),
            "@get all weapon from my disc"
        );
        assert_eq!(send_put_noeq_in_disc(), "@put noeq in my disc");
    }
}
