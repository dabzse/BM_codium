# BM::Codium [AI | EDU]

[![Programozási nyelv](assets/rust-nyelv.svg)](https://www.rust-lang.org/)
[![Licenc: AGPL-3.0-or-later](assets/licenc_AGPLv3.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Kettős licenc: AGPLv3+ / Kereskedelmi](assets/licenc_AGPLv3+_kereskedelmi.svg)](LICENSE)

---

## Angol nyelvű leírás

For the English documentation, please refer to [README.md](README.md)!

---

## Kezdetek és az alapötlet

Tegnap a **BSc államvizsgám** sikeres volt, így ma elhatároztam, hogy építek valami igazán hasznosat az egyetemek és a diákok számára...  
Ennek neve: **EDU::Codium**. Ez egy olyan oktatási kódszerkesztő, amely nem csupán programozási nyelvek tanulására szolgál, hanem iskolai feladatokhoz, egyetemi laborokhoz és zárthelyi/vizsgafeladatok megoldására is kiváló.  
**AI-mentes**: a kódgenerálás szigorúan tiltva van, a kódkiegészítés pedig alapértelmezetten ki van kapcsolva, hogy a diákok valóban megtanuljanak programozni. Ezen felül tanári funkciókat is tartalmaz feladatok és kurzusok kezelésére.  
Olyan programozói környezetek inspirálták ezt az ötletet, mint a **Zed**, a **Lapce**, a **Lite-XL** és főleg a **VSCodium** (amelyhez a név is hasonlít).  

Mellette párhuzamosan készül az **AI::Codium**, amely a platform professzionális fejlesztői változata beépített, offline helyi LLM támogatással.

---

## Két különálló kiadás

| Tulajdonság            | 🎓 EDU::Codium (`edu-codium`)                                   | ✦ AI::Codium (`ai-codium`)                              |
| :--------------------- | :-------------------------------------------------------------- | :------------------------------------------------------ |
| **Célközönség**        | Egyetemek, középiskolák, diákok, oktatók, vizsgák               | Haladó szoftvermérnökök, kutatók                        |
| **Generatív AI**       | **Szigorúan letiltva** (nincs kódmásolás, valódi tanulás)       | **Engedélyezve** (Ghost-text kiegészítés, LLM chat)     |
| **Kódkiegészítés**     | **Oktató által szabályozható** (alapértelmezetten ki, Kite mód) | Teljes AI és LSP autocompletion                         |
| **Oktatói segédlet**   | **Doc Tutor és hibamagyarázó** (tippeket ad, nem a kész kódot)  | AI magyarázatok és refaktorálási javaslatok             |
| **Oktatói szabályzat** | `.codium-edu.toml` fájlon keresztül (vizsgamód, korlátozások)   | Szokásos felhasználói beállítások                       |
| **Bináris méret**      | **önálló natív futtatható fájl**                                | **önálló natív futtatható fájl**                        |
| **Indítási idő**       | **terv szerint szinte azonnali** (Rust, kicsi és gyors)         | **terv szerint szinte azonnali** (Rust, kicsi és gyors) |

---

## Felépítés és kezelőfelület (ergonómia)

A BM::Codium a modern IDE-k legjavát gyúrja egyetlen, sallangmentes és villámgyors szoftverbe:

1. **VS Code-ból ismert átláthatóság**:
   - **Activity Bar (Bal ikonsáv)**: Függőleges vektorgombok az Intéző, Keresés, Oktatói segédlet, Git, Adatbázis, stb. és beállítások eléréséhez, aktív állapotjelző csíkkal.
   - **Oldalsáv**: Összecsukható fájlböngésző mappa-fával és nyelv-specifikus ikonokkal.
   - **Tabos szerkesztő**: Többlapos fájlkezelés aktív/inaktív kontraszttal és módosítatlan/módosított állapotjelző ponttal (`●`).
   - **Alsó dokk**: Többfunkciós alsó panel Terminállal, Problémák/Diagnosztika nézettel és Doc Tutor tippekkel.
   - **Státuszsor**: Letisztult alsó sáv kurzor koordinátákkal (`Ln, Col`), kódolással (`UTF-8`), szóközökkel és nyelvfelfedezéssel.

2. **JetBrains kényelmi funkciók**:
   - **Keresés bárhol (`Shift + Shift` / `Ctrl + P`)**: Középső keresősáv a fejlécben, amely fájlokban, szimbólumokban és parancsokban keres egyszerre.
   - **Futtató gomb (▶)**: Közvetlen fordítás és futtatás a fejlécből vagy közvetlenül a `main()` belépési pont mellől.
   - **Gyors dokumentáció (`F1` / Doc Tutor)**: Hozzáférés a szabványos könyvtári kártyákhoz és szintaxis példákhoz.

3. **100% natív Rust**:
   - **Zéró Electron / Zéró webes lomhaság**: Közvetlen gépi kód GPU hardveres gyorsítással.
   - **???**: valami, ami nagy fájlokat is akadásmentesen kezel.
   - **Vektoros SVG ikonok**: Beágyazott Codicons vektoros ikonok emojik helyett. (?)

4. **Kétnyelvű lokalizáció (Angol / Magyar)**:
   - **Alapértelmezett nyelv**: Angol (`EN`), megfelelve az egyetemi és nemzetközi szabványoknak.
   - **Párhuzamos magyar (`HU`) nyelv**: Azonnali váltási lehetőség egyetlen kattintással a fejlécben és a státuszsorban található `🌐 EN / HU` gombon. (?)

---

## Projekt architektúra

A projekt tiszta, moduláris Cargo Workspace szerkezetet követ:

```text
native-codium/
├── Cargo.toml                # Workspace definíció és optimalizált profilok
├── .cargo/config.toml        # Offline csomagkezelés (vendor-offline mappa)
├── crates/
│   ├── codium-core/          # Szövegszerkesztő-motor, Undo/Redo, Fájlfa, i18n nyelvi motor
│   ├── codium-edu/           # Oktatói szabályzat (.codium-edu.toml), DocTutor, hibamagyarázó
│   ├── codium-ai/            # Helyi LLM interfész, Ghost-text kódkiegészítő
│   └── codium-ui/            # Grafikus felület: címsáv, fő sáv, tabok, dokk
└── apps/
    └── codium-app/           # Indító binárisok:
        ├── src/bin/ai_codium.rs   -> ai-codium bináris
        └── src/bin/edu_codium.rs  -> edu-codium bináris
```

---

## Fordítás és futtatás

### Előfeltételek

- Rust fordító
- Szabványos Linux grafikus könyvtárak (Wayland / X11)

### Fordítás (Offline)

```bash
cd native-codium
cargo build --offline --bin edu-codium --bin ai-codium
```

A kész binárisok a `native-codium/target/debug/` mappába kerülnek (vagy `target/release/` a végleges optimalizált változathoz):

- `target/debug/edu-codium`
- `target/debug/ai-codium`

### Indítás

```bash
# Egyetemi oktatói változat indítása
./target/debug/edu-codium

# Adott fájl vagy mappa megnyitása
./target/debug/edu-codium /utvonal/a/projekthez

# AI fejlesztői változat indítása
./target/debug/ai-codium
```

---

## Egyetemi szabályzat konfiguráció (`.codium-edu.toml`)

Az oktatók a feladat-adattárak gyökerébe helyezhetnek egy `.codium-edu.toml` konfigurációt:

```toml
# Intézmény / Kurzus megnevezése
institution_name = "Programozás alapjai - BSc"

# Generatív AI kódírás szigorú tiltása
allow_ai_generation = false

# Oktató által engedélyezett token kiegészítés
allow_kite_completion = false

# Oktatói segédlet és hibatippek megjelenítése
doc_tutor_enabled = true

# Offline vizsgamód (hálózati hozzáférés letiltása)
exam_mode = true
```
