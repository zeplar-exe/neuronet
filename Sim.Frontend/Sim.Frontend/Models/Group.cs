using System;
using System.Collections.ObjectModel;
using Avalonia.Media;

namespace Sim.Frontend.Models;

public class Group : INode
{
    private static string[] Gods =
    [
        "Aphrodite", "Apollo", "Ares", "Artemis", "Athena", "Demeter", "Dionysus", "Hades", "Hephaestus",
        "Hera", "Hermes", "Hestia", "Poseidon", "Zeus", "Achlys", "Aether", "Aion", "Ananke", "Chaos",
        "Chronos", "Erebus", "Eros", "Gaia", "Hemera", "Hypnos", "Nemesis", "Nesoi", "Nyx", "Ourea",
        "Phanes", "Pontus", "Tartarus", "Thalassa", "Thanatos", "Uranus", "Coeus", "Crius", "Cronus", "Hyperion",
        "Iapetus", "Mnemosyne", "Oceanus", "Phoebe", "Rhea", "Tethys", "Theia", "Themis", "Asteria",
        "Astraeus", "Atlas", "Dione", "Helios", "Selene", "Eos", "Epimetheus", "Lelantos", "Leto",
        "Menoetius", "Metis", "Pallas", "Perses", "Prometheus", "Styx", "Adephagia", "Adikia", "Aergia",
        "Agathodaemon", "Agon", "Aidos", "Aisa", "Alala", "Alastor", "Aletheia", "Achos", "Ania", "Lupe",
        "Alke", "Amechania", "Amphilogiai", "Anaideia", "Androktasiai", "Angelia", "Apate", "Apheleia", "Arae",
        "Arete", "Atë", "Bia", "Caerus", "Corus", "Deimos", "Dikaiosyne", "Dike", "Dolos", "Dysnomia", "Dyssebeia",
        "Eiresione", "Ekecheiria", "Eleos", "Elpis", "Epiphron", "Eris", "Anteros", "Hedylogos", "Himeros", "Pothos", 
        "Eucleia", "Eulabeia", "Eupheme", "Eupraxia", "Eusebeia", "Euthenia", "Gelos", "Geras", "Harmonia",
        "Hedone", "Heimarmene", "Homados", "Homonoia", "Horkos", "Horme", "Hybris", "Hysminai", "Ioke", "Kakia",
        "Keres", "Koalemos", "Kratos", "Kydoimos", "Lethe", "Limos", "Litae", "Lyssa", "Machai", "Mania",
        "Clotho", "Lachesis", "Atropos", "Momus", "Moros", "Neikea", "Nike", "Nomos", "Oizys", "Oneiroi", "Palioxis", 
        "Peitharchia", "Peitho", "Penia", "Penthus", "Pepromene", "Pheme", "Philophrosyne", "Philotes", "Phobos", 
        "Phonoi", "Phrike", "Phthonus", "Pistis", "Poine", "Polemos", "Ponos",
        "Poros", "Praxidike", "Proioxis", "Prophasis", "Ptocheia", "Soter", "Soteria", "Sophrosyne", "Thrasos",
        "Tyche", "Zelos", "Amphiaraus", "Angelos", "Askalaphos", "Charon", "Erebos", "Alecto", "Tisiphone", "Megaera",
        "Hecate", "Aiakos", "Minos", "Rhadamanthys", "Keuthonymos", "Gorgyra", "Orphne", "Macaria",
        "Melinoe", "Menoetes", "Acheron", "Kokytos", "Eridanos", "Phlegethon", "Zagreus", "Aegaeon", "Amphitrite",
        "Benthesikyme", "Brizo", "Ceto", "Cymopoleia", "Eidothea", "Glaucus", "Leucothea", "Arethusa", "Dynamene", 
        "Galene", "Psamathe", "Thetis", "Nereus", "Nerites", "Idyia", "Palaemon", "Phorcys", "Pontos", "Proteus", 
        "Sangarius", "Actaeus", "Argyron", "Atabyrius", "Chalcon", "Chryson", "Damon", "Dexithea", "Lycos", "Lysagora",
        "Makelo", "Megalesius", "Mylas", "Nikon", "Ormenos", "Simon", "Skelmis", "Thaumas", "Thoosa", "Triteia",
        "Triton", "Tritones ", "Aeolus", "Alectrona", "Aparctias", "Apheliotes",
        "Argestes", "Boreas", "Caicias", "Circios", "Euronotus", "Eurus", "Lips", "Notus", "Skeiron", "Zephyrus",
        "Arke", "Astraios", "Stilbon", "Eosphorus", "Hesperus", "Pyroeis", "Phaethon", "Phaenon", "Aurai",
        "Aura", "Chione", "Ersa", "Hesperides", "Iris", "Men", "Nephele", "Pandia", "Alcyone", "Sterope",
        "Celaeno", "Electra", "Maia", "Merope", "Taygete", "Sabazios", "Zeus", "Aetna", "Amphictyonis", "Anthousai",
        "Aristaeus", "Attis", "Britomartis", "Aitnaios", "Alkon", "Eurymedon", "Onnes", "Tonnes", "Kelmis", "Skythes",
        "Chloris", "Comus", "Corymbus", "Cybele", "Acmon", "Damnameneus", "Delas", "Epimedes", "Heracles", "Iasios",
        "Titias", "Cyllenus", "Dryades", "Epimeliades", "Hamadryades", "Hecaterus", "Pyrrhichos", "Maenades",
        "Methe", "Meliae", "Daphne", "Metope", "Minthe", "Hekaerge", "Loxo", "Oupis", "Adrasteia", "Echo", "Ourea",
        "Palici", "Pan", "Achelous", "Acis", "Alpheus", "Asopus", "Cladeus", "Eurotas", "Nilus", "Peneus", "Scamander",
        "Priapus", "Silenus", "Telete", "Adonis", "Aphaea", "Cyamites", "Despoina", "Eunostus",
        "Persephone", "Philomelus", "Plutus", "Triptolemus", "Asclepius", "Aceso", "Aegle", "Epione", "Hygieia", "Iaso",
        "Paean", "Panacea", "Telesphorus", "Empusa", "Epiales", "Oneiroi", "Morpheus", "Acratopotes", "Agdistis",
        "Aphroditus", "Astraea", "Aglaea", "Euphrosyne", "Thalia", "Hegemone", "Antheia", "Pasithea", "Cleta",
        "Phaenna", "Eudaimonia", "Euthymia", "Calleis", "Paidia", "Pandaisia", "Pannychis", "Ceraon", "Chrysus",
        "Circe", "Syntribos", "Smaragos", "Asbetos", "Sabaktes", "Omodamos", "Deipneus", "Eileithyia", "Enyalius",
        "Enyo", "Epidotes", "Glycon", "Harpocrates", "Hebe", "Hermaphroditus", "Eunomia", "Eirene", "Thallo", "Auxo",
        "Karpo", "Pherousa", "Euporie", "Orthosie", "Auge", "Anatolia", "Musica", "Gymnastica", "Nymphe", "Mesembria",
        "Sponde", "Acte", "Hesperis", "Dysis", "Arktos", "Eiar", "Theros", "Pthinoporon", "Cheimon", "Hymenaios",
        "Ichnaea", "Iynx", "Matton", "Aoide", "Arche", "Melete", "Mneme", "Thelxinoe", "Calliope", "Clio", "Euterpe",
        "Erato", "Melpomene", "Polyhymnia ", "Terpsichore", "Urania", "Cephisso", "Apollonis", "Borysthenis",
        "Hypate", "Mese", "Nete", "Polymatheia", "Palaestra", "Rhapso", "Alexiares", "Anicetus", "Auxesia", "Damia",
    ];
    
    public string Name { get; set; }
    public Group? Parent { get; set; }
    public Color Color { get; set; }
    public ObservableCollection<INode> Children { get; } = [];

    public Group()
    {
        Name = Gods[Random.Shared.Next(Gods.Length)];
    }
}