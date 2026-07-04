# Monitor Systemu (HackerOS System Monitor)

Natywny monitor systemu dla Linuksa napisany w **Rust + GTK4**, inspirowany
[Mission Center](https://gitlab.com/mission-center-devs/mission-center).
Pokazuje na żywo: obciążenie procesora (ogólne i per-rdzeń), zużycie
pamięci RAM/swap, pojemność dysków, przepustowość sieci oraz sortowalną,
filtrowalną listę procesów.

Projekt skompilowano i przetestowano (uruchomienie pod Xvfb, brak paniki,
zero ostrzeżeń kompilatora) w tym środowisku na **Ubuntu 24.04 / GTK 4.14**;
architektura i zależności są identyczne dla **Debiana 12 (bookworm)** i
**Debiana 13 (trixie)**.

## Wymagania systemowe (Debian)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev cargo rustc
```

> Uwaga dot. wersji Rusta: pakiet `rustc` z Debiana 12 (1.63) jest za stary
> dla części zależności. Zalecane jest zainstalowanie toolchaina przez
> [rustup](https://rustup.rs) (`curl https://sh.rustup.rs -sSf | sh`), który
> da Ci aktualny stabilny Rust. Ten projekt buduje się od Rusta 1.75 wzwyż
> (dokładnie taką wersję zweryfikowano podczas budowy w tym środowisku).
> Jeśli używasz rustup, powyższy `Cargo.lock` możesz odświeżyć poleceniem
> `cargo update` — zostały w nim celowo przypięte nieco starsze wersje
> `hashbrown`/`indexmap`/`rayon`, potrzebne tylko dla toolchaina 1.75.

## Budowanie

```bash
cargo build --release
```

Binarka powstaje w `target/release/mission-monitor`.

## Uruchomienie

```bash
./target/release/mission-monitor
```

## Struktura projektu

```
src/
  main.rs              punkt wejścia
  app.rs                okno główne, pasek boczny, pętla odświeżania (1 Hz)
  stats.rs              model stanu systemu (sysinfo) + bufory historii
  graph.rs              lekki widget wykresu liniowego (DrawingArea + cairo)
  process_object.rs     GObject opakowujący wiersz procesu dla ColumnView
  style.css             arkusz stylów (ciemne karty, akcenty kolorystyczne)
  views/
    overview.rs          zakładka "Przegląd"
    cpu.rs                zakładka "Procesor" (ogólny + per-rdzeń)
    memory.rs             zakładka "Pamięć" (RAM + swap)
    disk.rs                zakładka "Dyski" (paski pojemności)
    network.rs             zakładka "Sieć" (przepustowość + interfejsy)
    processes.rs           zakładka "Procesy" (ColumnView, sortowanie, filtr)
resources/
  mission-monitor.desktop  wpis dla menu aplikacji
  icons/mission-monitor.svg
```

## Pakowanie do `.deb`

Najprostsza droga to `cargo-deb`:

```bash
cargo install cargo-deb
cargo deb            # tworzy target/debian/mission-monitor_0.1.0_amd64.deb
sudo dpkg -i target/debian/mission-monitor_*.deb
```

Metadane pakietu (zależności runtime, opis, instalowane pliki — binarka,
`.desktop`, ikona) są już zdefiniowane w `[package.metadata.deb]` w
`Cargo.toml`. Wygenerowany pakiet zależy od `libgtk-4-1`, więc na czystym
Debianie instaluje się przez zwykłe `apt`/`dpkg` bez dodatkowych kroków.

## Architektura / decyzje projektowe

- **Odświeżanie danych**: `glib::timeout_add_local` co 1000 ms odświeża
  `sysinfo::System`/`Disks`/`Networks` i przepycha nowe próbki do buforów
  pierścieniowych (`stats::History`), które napędzają wykresy.
- **Wykresy**: zwykły `gtk::DrawingArea` + `cairo`, bez subklasowania
  widgetu — mniej kodu, ta sama płynność.
- **Lista procesów**: `gtk::ColumnView` nad `gio::ListStore<ProcessObject>`,
  gdzie `ProcessObject` to lekki `glib::Object` (subclass) bez pełnego
  systemu property GObject — czytane są zwykłe gettery w fabrykach kolumn.
  Sortowanie i filtrowanie tekstowe działają natywnie przez
  `SortListModel`/`FilterListModel`.
- **UI**: pasek boczny (`ListBox` + ikony symboliczne z motywu Adwaita) i
  `gtk::Stack` z przejściem "crossfade" między zakładkami — układ 1:1 z
  ideą Mission Center (stały sidebar, karty z wykresami, tabela procesów).

## Znane ograniczenia / dalszy rozwój

- Brak wykresu GPU (Mission Center czyta go przez własną bibliotekę C;
  dodanie wymagałoby integracji z `nvml`/`vulkan`/`drm`, celowo pominięte
  dla przejrzystości kodu bazowego).
- Kolumna „Użytkownik” w tabeli procesów jest obecnie pusta na Linuksie,
  ponieważ `sysinfo` zwraca tylko surowe `Uid`; można ją uzupełnić,
  mapując `Uid` na nazwę przez `sysinfo::Users`.
- Zamykanie/zabijanie procesu z poziomu UI nie jest zaimplementowane —
  łatwo dodać jako akcję kontekstową na wierszu `ColumnView`
  (`process.kill()` z `sysinfo`).
