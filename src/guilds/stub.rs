//! Empty guild implementations for catalog entries without commands yet.

macro_rules! declare_stub_guild {
    ($mod_name:ident, $struct_name:ident) => {
        mod $mod_name {
            use crate::command::Command;
            use crate::guilds::Guild;
            use crate::triggers::Trigger;
            use std::collections::HashMap;

            #[derive(Default)]
            pub struct $struct_name;

            impl Guild for $struct_name {
                fn commands(&self) -> HashMap<String, Command> {
                    HashMap::new()
                }

                fn triggers(&self) -> Vec<Trigger> {
                    Vec::new()
                }
            }
        }
        pub use $mod_name::$struct_name;
    };
}

declare_stub_guild!(alchemists, AlchemistsGuild);
declare_stub_guild!(archers, ArchersGuild);
declare_stub_guild!(bard, BardGuild);
declare_stub_guild!(beastmaster, BeastmasterGuild);
declare_stub_guild!(cavalier, CavalierGuild);
declare_stub_guild!(civilized, CivilizedGuild);
declare_stub_guild!(civilized_fighters, CivilizedFightersGuild);
declare_stub_guild!(crimson, CrimsonGuild);
declare_stub_guild!(druids, DruidsGuild);
declare_stub_guild!(evil_religious, EvilReligiousGuild);
declare_stub_guild!(explorer, ExplorerGuild);
declare_stub_guild!(inf, InfGuild);
declare_stub_guild!(knight, KnightGuild);
declare_stub_guild!(magical, MagicalGuild);
declare_stub_guild!(merchant, MerchantGuild);
declare_stub_guild!(navigator, NavigatorGuild);
declare_stub_guild!(nomad, NomadGuild);
declare_stub_guild!(nun, NunGuild);
declare_stub_guild!(runemages, RunemagesGuild);
declare_stub_guild!(sailor, SailorGuild);
declare_stub_guild!(squire, SquireGuild);
declare_stub_guild!(tarmalen, TarmalenGuild);
declare_stub_guild!(templar, TemplarGuild);
declare_stub_guild!(treenav, TreenavGuild);
