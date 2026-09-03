# BM::Codium [AI | EDU]

[![Programozási nyelv](assets/rust-nyelv.svg)](https://www.rust-lang.org/)
[![GUI keretrendszer](assets/gui_floem_gpu_gyorsitott.svg)](https://github.com/lapce/floem)
[![Licenc: AGPL-3.0-or-later](assets/licenc_AGPLv3.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Kettős licenc: AGPLv3+ / Kereskedelmi](assets/licenc_AGPLv3+_kereskedelmi.svg)](LICENSE)

> Villámgyors, hardveresen gyorsított natív Rust fejlesztői környezet (IDE), amely egyesíti a **VS Code** megszokott ergonómiáját, a **Zed** hardveresen gyorsított sebességét, a **Floem** tiszta reaktív architektúráját és a **JetBrains** legkedveltebb produktivitási funkcióit (Mindent kereső / Search Everywhere, Futtatás a margóról / Run Gutter).

---

## Angol nyelvű leírás

For the English documentation, please refer to [README.md](README.md)!

---

## Fejlesztési státusz

| Jellemző                   | Részletek                                                                           |
| :------------------------- | :---------------------------------------------------------------------------------- |
| **Architektúra**           | 100% tiszta Rust (`BM_codium` workspace), Floem 0.2 GPU motor, Xi-Rope szövegpuffer |
| **Telemetria és felesleg** | **0 % telemetria, 0 % Electron / böngészőmotor, 100% offline vendor függetlenség**  |
| **Elsődleges platform**    | **Linux (Wayland & X11)** (Tesztelt és működőképes); Windows és macOS tervezve      |

---

## Kezdetek és az alapötlet

**2026. június 12-én** volt a **BSc záróvizsgám**, így a rá következő napon elhatároztam, hogy építek valami igazán hasznosat az egyetemek és a diákok számára...  
Ennek neve: **EDU::Codium**. Ez egy kifejezetten oktatási célú kódszerkesztő, nem csupán a programozási nyelvek elsajátítására, hanem iskolai feladatok, laborok, egyetemi beadandók és nem utolsó sorban **zárthelyi és vizsgafeladatok lebonyolítására**.  
Teljesen **AI-mentes**. A kódkiegészítés alapértelmezetten tiltott, hogy a hallgatók valóban megtanuljanak programozni. Továbbá olyan funkciókat is tartalmaz, amelyekkel az oktatók feladatokat és kurzusokat tudnak kitűzni.  
A **Floem** GUI keretrendszerre épül, és olyan környezetek inspirálták, mint a **Zed**, a **Lapce**, a **Lite-XL** valamint legfőképpen a **VSCodium** (amelyhez a név is hasonló).  

Ezzel párhuzamosan készül az **AI::Codium**, amely a haladó/professzionális szoftvermérnököknek szánt kiadás beépített, offline helyi LLM képességekkel.

---

## Két célorientált kiadás

| Tulajdonság          | 🎓 EDU::Codium (`edu-codium`)                                             | ✦ AI::Codium (`ai-codium`)                              |
| :------------------- | :------------------------------------------------------------------------ | :------------------------------------------------------ |
| **Célközönség**      | Egyetemek, középiskolák, hallgatók, oktatók, vizsgák                      | Professzionális fejlesztők, kutatók                     |
| **Generatív AI**     | **Szigorúan tiltott** (csalás megelőzése, valódi tanulás kikényszerítése) | **Engedélyezett** (Ghost-text kódkiegészítés)           |
| **Kódkiegészítés**   | **Tanár által konfigurálható** (alapértelmezetten kikapcsolva, Kite mód)  | Teljes AI és LSP kódkiegészítés                         |
| **Oktatói segédlet** | **Doc Tutor és hibamagyarázó** (tippeket ad és magyaráz, nem kész kódot   | AI magyarázatok és refaktorálási javaslatok             |
| **Oktatói házirend** | `.codium-edu.toml` fájlon keresztül szabályozható (vizsgamód)             | Szokásos felhasználói beállítások                       |
| **Bináris méret**    | **önálló natív futtatható fájl**                                          | **önálló natív futtatható fájl**                        |
| **Indítási idő**     | **terv szerint szinte azonnali** (Rust, kicsi és gyors)                   | **terv szerint szinte azonnali** (Rust, kicsi és gyors) |

---

## Támogatott programozási nyelvek

Az egyetemi informatikai képzések, mérnöki kurzusok és a professzionális szoftverfejlesztés igényeire hangolva **17 kulcsfontosságú programozási nyelv**, valamint a mindennapi statikus web, serverless-web, stíluslapok, „olvass-el” fájlok/könyvek, parancshéj és konfigurációs formátumok közvetlen támogatásával:

| Programozási nyelv / Technológia | Kategória                                  | Fájlkiterjesztések                     | Ajánlott LSP szerver               | Margó futtatás (▶) és Doc Tutor                                                                 |
| :------------------------------- | :----------------------------------------- | :------------------------------------- | :--------------------------------- | :---------------------------------------------------------------------------------------------- |
| **Python**                       | Dinamikus / Szkript                        | `.py`, `.pyw`                          | `pyright` / `jedi-language-server` | Közvetlen értelmező futtatás (`python3`), beépített I/O és kontextuskezelő puskák.              |
| **PHP**                          | Dinamikus / Web                            | `.php`, `.phtml`                       | `intelephense` / `phpactor`        | CLI szkriptfuttatás (`php`), tömbkezelési és webes háttérrendszer segédletek.                   |
| **Rust**                         | Rendszerprogramozás / Natív                | `.rs`                                  | `rust-analyzer`                    | `cargo run` integráció, memóriabiztonsági és kölcsönzési puskák, `Result`/`Option` hibakezelés. |
| **C**                            | Rendszerprogramozás / Alacsony szint       | `.c`, `.h`                             | `clangd`                           | `gcc`/`clang` fordítás és futtatás, mutatóaritmetika, dinamikus memória (`malloc`/`free`).      |
| **C++**                          | Rendszerszintű / OOP                       | `.cpp`, `.hpp`, `.cc`, `.cxx`          | `clangd`                           | `g++`/`clang++` futtatás, RAII, STL tárolók, okosmutatók (smart pointers).                      |
| **C#**                           | Vállalati / .NET                           | `.cs`                                  | `OmniSharp` / `csharp-ls`          | `dotnet run`, LINQ szintaxis, OOP tervezés, async/await aszinkron minták.                       |
| **Java**                         | Vállalati / JVM                            | `.java`                                | `jdtls` (Eclipse JDT)              | `javac` & `java` futtató, OOP alapelvek, kollekció keretrendszer.                               |
| **Kotlin**                       | JVM / Modern alkalmazás                    | `.kt`, `.kts`                          | `kotlin-language-server`           | `kotlinc` fordító, null-biztonság, adat-osztályok (data class), coroutines.                     |
| **Erlang**                       | Funkcionális / Konkurens                   | `.erl`, `.hrl`                         | `erlang_ls`                        | `erlc` fordítás és `erl` shell, aktor modell, mintára illesztés, üzenetküldés.                  |
| **Scala**                        | JVM / Funkcionális OOP                     | `.scala`, `.sc`                        | `metals`                           | `scala` CLI futtatás, immutábilis kollekciók, pattern matching, traits.                         |
| **Elixir**                       | Funkcionális / BEAM                        | `.ex`, `.exs`                          | `elixir-ls` / `expert`             | `elixir` futtatás, csővezeték operátor (`\|>`), OTP felügyeleti fák, hibatűrés.                 |
| **Go**                           | Rendszer / Felhő natív                     | `.go`                                  | `gopls`                            | `go run` futtatás, goroutinok és csatornák (channels) párhuzamossága, interfész szerződések.    |
| **TypeScript**                   | Web / Statikusan típusos JS                | `.ts`, `.tsx`                          | `typescript-language-server`       | `tsx`/`node` futtatás, típusinferencia, interfészek, generikus típusok puskái.                  |
| **JavaScript**                   | Serverless-web (JS) / Szkript              | `.js`, `.mjs`, `.cjs`                  | `quick_lint_js` / `ts-server`      | `node`/`yarn`/`pnpm`/`bun`/`deno` futtatás, async/await, DOM API és fetch() puskák.             |
| **Ruby**                         | Dinamikus / Szkript & Web                  | `.rb`, `.rake`, `Gemfile`              | `solargraph` / `ruby-lsp`          | `ruby` futtatás, blokkok, gemek, metaprogramozás és Rails segédletek.                           |
| **Zig**                          | Rendszerprogramozás / Modern C alternatíva | `.zig`                                 | `zls`                              | `zig run` / `zig build`, fordításidejű kód (`comptime`), manuális memória.                      |
| **Swift**                        | Natív / Apple & Szerveroldali              | `.swift`                               | `sourcekit-lsp`                    | `swift run` / `swift`, ARC memóriakezelés, opciós típusok, protokollok.                         |
| **Objective-C**                  | Natív / Legacy macOS & iOS                 | `.m`, `.mm`                            | `clangd`                           | `clang` fordítás, üzenetküldés (`[target action]`), régebbi Cocoa kompatibilitás.               |
| **HTML / XML**                   | Statikus web / Jelölőnyelv                 | `.html`, `.htm`, `.xml`, `.svg`        | `html-languageserver` / `lemminx`  | Beépített előnézet, DOM fa struktúra és nyitó/záró címke puskák.                                |
| **CSS / SCSS**                   | Stíluslapok / Web design                   | `.css`, `.scss`, `.sass`, `.less`      | `css-languageserver`               | Flexbox, CSS Grid és változók puskái, élő színminta előnézet.                                   |
| **Markdown & TeX**               | „Olvass-el” fájlok / Könyvek               | `.md`, `.rst`, `.asc`, `.tex`          | `marksman` / `texlab`              | Élő Markdown előnézet, LaTeX képletfordítás, műszaki dokumentációs segédletek.                  |
| **Shell & Szkriptek**            | Parancshéj (shell, batch, cmd, ps1)        | `.sh`, `.bash`, `.bat`, `.cmd`, `.ps1` | `bash-language-server`             | Közvetlen terminál futtatás (▶), POSIX Shell, Windows Batch és PowerShell puskák.               |
| **Konfiguráció & SQL**           | Adatbázis & Konfiguráció                   | `.toml`, `.sql`, `.ini`, `.json`       | `taplo` / `sqls` / `json-ls`       | SQL lekérdezés-futtatás, sémavalidáció, kulcs-érték és formázási puskák.                        |

---

## Támogatott platformok

| Platform    | Megjelenítő szerver / Grafika          | Státusz                 | Megjegyzés                                                                                                             |
| :---------- | :------------------------------------- | :---------------------- | :--------------------------------------------------------------------------------------------------------------------- |
| **Linux**   | Wayland & X11 (WGPU / Vulkan / OpenGL) | **Támogatott (Tier 1)** | Elsődleges fejlesztési célpont. Hardveresen gyorsított GPU renderelés, tesztelve Wayland és X11 asztali környezeteken. |
| **Windows** | Win32 / DirectX 12 / Vulkan            | *Tervezve (Tier 2)*     | A Floem keresztplatformos ablakkészlete felkészítve; csomagolás előkészítés alatt.                                     |
| **macOS**   | Metal / Cocoa                          | *Tervezve (Tier 2)*     | Apple Silicon (M-széria) és x86_64 natív build célpontok.                                                              |

---

## Elrendezés és ergonómia (Felhasználói felület)

A *::Codium az ismert és bevált IDE megoldásokat ötvözi egy egységes, feleslegektől mentes felületen:

1. **VS Code-ból ismert elemek**:
   - **Tevékenységsáv (Activity Bar)**: Bal oldali függőleges vektorsáv (Fájlkezelő, Keresés, Doc Tutor, Beállítások) aktív jelölőcsíkkal.
   - **Oldalsáv (Sidebar)**: Összecsukható fájlfa projektstruktúrával és nyelvspecifikus ikonokkal.
   - **Füles szerkesztő (Tabbed Editor)**: Többlapos, felosztható dokumentumfülek piszkos állapotjelző ponttal (`●`).
   - **Alsó dokk (Bottom Dock)**: Többfunkciós alsó panel Terminállal, Problémák/Diagnosztika nézettel és Doc Tutor kimenettel.
   - **Állapotsor (Status Bar)**: Letisztult lábléc kurzorpozícióval (`Sor, Oszlop`), Kódolással (`UTF-8`), Szóköz behúzással és Aktív nyelvvel.

2. **JetBrains produktivitási funkciók**:
   - **Mindent kereső (`Shift + Shift` / `Ctrl + P`)**: Központi keresősáv a címsorban fájlok, szimbólumok és parancsok azonnali eléréséhez.
   - **Futtatás gomb a margón (▶)**: Közvetlen fordítás és indítás a címsorból vagy a belépési pontok (`main()`) mellől.
   - **Gyors dokumentáció (`F1` / Doc Tutor)**: Azonnali hozzáférés a standard könyvtári puskákhoz és szintaxis kártyákhoz.

3. **100% tiszta Rust és Floem**:
   - **Zéró Electron / Zéró Webes lomhaság**: Közvetlen gépi kód, GPU hardveres gyorsítással (WGPU és Taffy flexbox motor).
   - **Xi-Rope szövegpuffer**: B-fa alapú memóriakezelés, amely gigabájtos fájloknál is akadásmentes szerkesztést biztosít.
   - **Vektoros SVG ikonok**: Beágyazott, éles Codicons vektoros ikonok emojik helyett. (?)

4. **Kétnyelvű lokalizáció (EN / HU)**:
   - **Alapértelmezett nyelv**: Angol (`EN`) a nemzetközi akadémiai szabványnak megfelelően.
   - **Párhuzamos magyar (`HU`) támogatás**: Azonnali futásidejű nyelvváltás a címsor és állapotsor `🌐 EN / HU` gombjával. (?)

---

## A projekt felépítése

A projekt moduláris Rust Cargo munkaterületként (workspace) épül fel:

```text
BM_codium/
├── Cargo.toml                # Munkaterület definíció és optimalizált profilok
├── .cargo/config.toml        # Offline vendored csomagfeloldás (vendor-offline)
├── crates/
│   ├── codium-core/          # Xi-Rope puffer, visszavonás/újra, fájlfa, i18n lokalizáció, Nyelv definíciók (13 nyelv)
│   ├── codium-edu/           # Egyetemi házirendek (.codium-edu.toml), DocTutor, hibamagyarázó
│   ├── codium-ai/            # Helyi LLM traitek, ghost-text kódkiegészítő szolgáltató
│   └── codium-ui/            # Floem nézetek: címsor, tevékenységsáv, oldalsáv, fülek, alsó dokk
└── apps/
    └── codium-app/           # Futtatható binárisok:
        ├── src/bin/ai_codium.rs   -> ai-codium futtatható állomány (~29 MB)
        └── src/bin/edu_codium.rs  -> edu-codium futtatható állomány (~29 MB)
```

---

## Fordítás és futtatás

### Előfeltételek

- Rust fordító (`rustc >= 1.86`, tesztelve: `rustc 1.95.0`)
- Standard Linux GUI könyvtárak (Wayland / X11, standard C könyvtár)

### Binárisok fordítása (Offline)

```bash
cargo build --offline --bin edu-codium --bin ai-codium
```

Az elkészült binárisok a `target/debug/` mappába kerülnek (vagy `target/release/` a véglegesített változathoz):

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
institution_name = "Informatikai Tanszék - BSc"

# Generatív AI kódírás szigorú tiltása
allow_ai_generation = false

# Oktató által engedélyezett token kiegészítés
allow_kite_completion = false

# Oktatói segédlet és hibatippek megjelenítése
doc_tutor_enabled = true

# Offline vizsgamód (hálózati hozzáférés letiltása)
exam_mode = true
```

---

## Visszajelzés és közreműködés

Örömmel fogadjuk az egyetemi oktatók, hallgatók és nyílt forráskódú fejlesztők javaslatait és visszajelzéseit:

- **Egyetemi tantervek és vizsgaszabályzatok**: Ha oktatóként érdekel egyedi vizsgamód vagy intézményi `.codium-edu.toml` szabályzat, oszd meg velünk a tanszéki követelményeket.
- **Doc Tutor puskák és oktatókártyák**: Segíts bővíteni a 13 támogatott nyelvhez tartozó offline szintaxis puskákat és fordítóhiba-magyarázatokat.
- **Ergonómia és teljesítmény**: Oszd meg véleményedet a billentyűparancsokról, a Mindent kereső működéséről vagy a margóról indítható futtatásról.
