//! Hand-written synthetic Spanish test data. No words, rules or suggestion tables were copied
//! from a system dictionary. This is a small fixture, not a redistributable Spanish dictionary.

pub const AFF: &str = "SET UTF-8
FLAG UTF-8
FORBIDDENWORD F
TRY añeiorsutclmpbgd
KEY qw|as|zx
MAP 1
MAP nñ
REP 1
REP u v
SFX A Y 1 # synthetic -s
SFX A 0 s .
";

pub const DIC: &str = "16
casa/A
perro/A
gato/A
escuela
libro
niño
mañana
teórico
práctico
presidente
grande
pequeño
hablar
ex
ONU
zqprohibida/F
";

pub fn install() {
    if let Err(e) = super::install_dictionary(AFF, DIC) {
        panic!("test dictionary: {e}");
    }
}
