use hyper;
use json; // TODO A vérifier
extern crate stopwatch;
use stopwatch::{Stopwatch};

use track::*;

pub mod Worker {
    // Modélise les champs de l'objet Agent qui va effectuer le téléchargement
    struct Agent {
        compte : String,
        nbPages : i16,
        Limite : i16,
    }

    // Modélise les fonctions que l'objet Agent aura pour effectuer le téléchargement
    impl Agent {
        fn initialisation_session() -> bool {

        }

        fn start() {

        }

        fn nettoyage_nom(String nom_fichier) -> String {

        }

        // fn essai_requete() TODO A voir si pas déjà implémentée ailleurs
        fn recuperation_liste_sons(track Piste) -> String {

        }

        fn telechargement_url(track Piste) -> String {

        }

        fn telechargement_piste(track Piste) -> Stream {

        }
    }
}
