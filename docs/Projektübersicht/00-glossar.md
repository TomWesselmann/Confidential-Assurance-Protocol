# 00 - Umfangreiches Vokabelbuch

## 📖 Über dieses Glossar

Dieses Vokabelbuch ist Ihr **Nachschlagewerk für alle Fachbegriffe** aus dem LsKG-Agent Projekt. Es ist thematisch gegliedert und zeigt Zusammenhänge zwischen Begriffen.

**Nutzungshinweise:**
- 🔤 **Für jeden Begriff:** Einfache Erklärung + Technische Definition + Analogie
- 🔗 **Querverweise:** Verwandte Begriffe werden verlinkt
- 📊 **Thematische Gliederung:** Nach Fachgebieten sortiert
- ⭐ **Wichtigkeit:** Kern-Begriffe sind markiert

**Navigation:**
- Verwenden Sie die Themen-Übersicht unten für schnellen Zugriff
- Oder nutzen Sie die Browser-Suche (Strg+F / Cmd+F)

---

## 📑 Inhaltsverzeichnis

1. [LkSG & Compliance](#1-lksg--compliance) (8 Begriffe)
2. [Kryptographie & Sicherheit](#2-kryptographie--sicherheit) (20 Begriffe)
3. [Blockchain & Distributed Ledger](#3-blockchain--distributed-ledger) (6 Begriffe)
4. [Software-Architektur](#4-software-architektur) (15 Begriffe)
5. [APIs & Schnittstellen](#5-apis--schnittstellen) (12 Begriffe)
6. [Datenformate & Serialisierung](#6-datenformate--serialisierung) (8 Begriffe)
7. [Datenbank & Storage](#7-datenbank--storage) (10 Begriffe)
8. [Deployment & Container](#8-deployment--container) (12 Begriffe)
9. [Netzwerk & Kommunikation](#9-netzwerk--kommunikation) (10 Begriffe)
10. [Authentifizierung & Autorisierung](#10-authentifizierung--autorisierung) (9 Begriffe)
11. [Testing & Quality Assurance](#11-testing--quality-assurance) (8 Begriffe)
12. [Monitoring & Observability](#12-monitoring--observability) (7 Begriffe)
13. [Rust-spezifische Begriffe](#13-rust-spezifische-begriffe) (12 Begriffe)
14. [WASM & WebAssembly](#14-wasm--webassembly) (5 Begriffe)
15. [Proof-Systeme](#15-proof-systeme) (8 Begriffe)
16. [Allgemeine IT-Begriffe](#16-allgemeine-it-begriffe) (15 Begriffe)

**Gesamt: 165+ Begriffe**

---

## 1. LkSG & Compliance

### LkSG (Lieferkettensorgfaltspflichtengesetz) ⭐
**Einfach:** Deutsches Gesetz seit 2023, das große Unternehmen verpflichtet, ihre Lieferketten auf Menschenrechts- und Umweltverstöße zu prüfen.

**Technisch:** Bundesgesetz zur Einhaltung menschenrechtlicher und umweltbezogener Sorgfaltspflichten in Lieferketten. Gilt für Unternehmen mit > 1.000 (ab 2024) bzw. > 3.000 (ab 2023) Mitarbeitern in Deutschland.

**Analogie:** Wie ein TÜV für Lieferketten - Unternehmen müssen regelmäßig prüfen und dokumentieren.

**Im Projekt:** Das LsKG ist der Hauptanwendungsfall - unser System hilft bei der Erfüllung dieser Pflicht.

**Verwandte Begriffe:** [Compliance](#compliance), [UBO](#ubo-ultimate-beneficial-owner), [Supply Chain](#supply-chain-lieferkette)

---

### UBO (Ultimate Beneficial Owner) ⭐
**Einfach:** Die "wahren" Eigentümer eines Unternehmens - natürliche Personen, die letztendlich wirtschaftlich berechtigt sind.

**Technisch:** Natürliche Person, die mehr als 25% der Anteile oder Stimmrechte hält oder auf andere Weise Kontrolle ausübt (nach § 3 Abs. 1 GwG).

**Analogie:** Wie der Eigentümer eines Hauses, auch wenn es auf eine Firma eingetragen ist - am Ende steht eine echte Person dahinter.

**Im Projekt:** UBO-Daten werden in `ubos.csv` importiert und als vertrauliche Informationen behandelt (Merkle-Root statt Rohdaten).

**Datenstruktur:**
```rust
struct Ubo {
    name: String,           // Vollständiger Name
    birthdate: String,      // RFC3339 Format
    citizenship: String,    // Staatsangehörigkeit (ISO Code)
}
```

**Verwandte Begriffe:** [PII](#pii-personally-identifiable-information), [Privacy by Design](#privacy-by-design), [Zero-Knowledge](#zero-knowledge-proof)

---

### Compliance
**Einfach:** Die Einhaltung von Gesetzen, Regeln und Standards.

**Technisch:** Organizational conformity with legal requirements, industry standards, and internal policies.

**Analogie:** Wie Verkehrsregeln befolgen - man muss die Vorschriften kennen und einhalten.

**Im Projekt:** Das System erstellt Compliance-Nachweise (Proofs), die belegen, dass Regeln eingehalten wurden.

**Verwandte Begriffe:** [Policy](#policy), [Audit Trail](#audit-trail), [Verification](#verification)

---

### Supply Chain (Lieferkette)
**Einfach:** Das Netzwerk aller Unternehmen und Personen, die an der Herstellung und Lieferung eines Produkts beteiligt sind.

**Technisch:** Network of suppliers, manufacturers, distributors, and retailers involved in producing and delivering a product.

**Analogie:** Wie eine Kette vom Baumwollfeld über die Weberei, Färberei, Näherei bis zum Kleidungsgeschäft.

**Im Projekt:** Lieferanten werden in Tiers organisiert (TIER_1, TIER_2, etc.) und in `suppliers.csv` verwaltet.

**Datenstruktur:**
```rust
struct Supplier {
    name: String,
    jurisdiction: String,   // Land (ISO Code)
    tier: String,          // TIER_1, TIER_2, ...
}
```

**Verwandte Begriffe:** [Tier](#tier), [Supplier](#supplier)

---

### Policy
**Einfach:** Ein Regelwerk, das definiert, welche Bedingungen erfüllt sein müssen (z.B. "maximal 100 Lieferanten").

**Technisch:** Machine-readable compliance ruleset defining constraints and validation criteria.

**Analogie:** Wie ein Bauplan mit Spezifikationen - "Das Haus muss mindestens 2 Ausgänge haben".

**Im Projekt:** Policies werden als YAML-Dateien definiert und in PolicyV2-Format kompiliert.

**Beispiel:**
```yaml
version: lksg.v1
name: "Standard Policy"
constraints:
  require_at_least_one_ubo: true
  supplier_count_max: 100
```

**Verwandte Begriffe:** [PolicyV2](#policyv2), [Constraint](#constraint), [Statement](#statement)

---

### Due Diligence (Sorgfaltspflicht)
**Einfach:** Die gebotene Sorgfalt bei der Prüfung von Geschäftspartnern und Lieferketten.

**Technisch:** Systematic process of investigation and risk assessment before entering business relationships.

**Analogie:** Wie eine Bonitätsprüfung vor einem Kredit - man prüft, mit wem man Geschäfte macht.

**Im Projekt:** Das System unterstützt Due Diligence durch automatisierte Prüfungen gegen Sanktionslisten.

**Verwandte Begriffe:** [Risk Assessment](#risk-assessment), [Sanctions Lists](#sanctions-lists)

---

### Tier
**Einfach:** Ebene in der Lieferkette - Tier 1 sind direkte Lieferanten, Tier 2 deren Lieferanten, usw.

**Technisch:** Hierarchical level in supply chain structure, numbered from direct suppliers (Tier 1) outward.

**Analogie:** Wie Generationen in einem Stammbaum - Kinder, Enkel, Urenkel.

**Im Projekt:** Wird als String-Feld in `Supplier` gespeichert: "TIER_1", "TIER_2", etc.

**Verwandte Begriffe:** [Supply Chain](#supply-chain-lieferkette), [Supplier](#supplier)

---

### Jurisdiction
**Einfach:** Der rechtliche Zuständigkeitsbereich, meist ein Land oder eine Region.

**Technisch:** Geographic area or legal domain with specific laws and regulations.

**Analogie:** Wie Bundesländer in Deutschland - verschiedene Zuständigkeiten und Regeln.

**Im Projekt:** ISO-Ländercodes (DE, FR, US, etc.) zur Identifikation des Sitzlandes von Lieferanten.

**Verwandte Begriffe:** [Sanctions Lists](#sanctions-lists), [High-Risk Country](#high-risk-country)

---

## 2. Kryptographie & Sicherheit

### Hash / Hash-Funktion ⭐
**Einfach:** Ein digitaler "Fingerabdruck" für Daten - jede kleinste Änderung erzeugt einen komplett anderen Hash.

**Technisch:** Mathematical one-way function that maps arbitrary-length input to fixed-length output (digest). Collision-resistant and deterministic.

**Analogie:** Wie eine Quersumme, die nur für genau diese Daten passt - ändert man ein Zeichen, ändert sich die komplette Quersumme.

**Eigenschaften:**
- **Einweg:** Aus dem Hash kann man nicht die Original-Daten rekonstruieren
- **Deterministisch:** Gleiche Eingabe → gleicher Hash
- **Kollisionsresistent:** Praktisch unmöglich, zwei verschiedene Eingaben mit gleichem Hash zu finden

**Im Projekt:** Verwendet BLAKE3 für Merkle-Roots und SHA3-256 für Hashes.

**Verwandte Begriffe:** [BLAKE3](#blake3), [SHA3-256](#sha3-256), [Merkle Tree](#merkle-tree)

---

### BLAKE3 ⭐
**Einfach:** Eine sehr schnelle und sichere Hash-Funktion (neuer als SHA-256).

**Technisch:** Cryptographic hash function based on BLAKE2, optimized for speed (parallel computation) while maintaining high security. Output: 256 bits.

**Analogie:** Wie ein Hochgeschwindigkeits-Stempel, der sehr komplexe Siegel erstellt.

**Vorteile:**
- **Schnell:** Bis zu 10x schneller als SHA-256
- **Parallel:** Kann mehrere CPU-Kerne nutzen
- **Sicher:** Keine bekannten Schwachstellen

**Im Projekt:** Standard für alle Merkle-Root-Berechnungen (Supplier-Root, UBO-Root).

**Code-Beispiel:**
```rust
use blake3::hash;
let hash = hash(b"Hello World");
println!("0x{}", hash.to_hex());
```

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [Merkle Tree](#merkle-tree), [Commitment](#commitment)

---

### SHA3-256 ⭐
**Einfach:** Eine standardisierte Hash-Funktion (256 Bit), die als besonders sicher gilt.

**Technisch:** Standardized cryptographic hash function (NIST FIPS 202), based on Keccak algorithm. Output: 256 bits (64 hex characters).

**Analogie:** Wie ein offiziell zugelassener Stempel - gilt als Standard.

**Im Projekt:** Verwendet für:
- Manifest-Hashes
- Proof-Hashes
- Policy-Hashes
- Audit Chain (Hash-Kette)

**Format:** `0x` + 64 Hexadezimal-Zeichen
```
0x83a8779ddef4567890abcdef1234567890abcdef1234567890abcdef12345678
```

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [Digest](#digest)

---

### Ed25519 ⭐
**Einfach:** Ein Verfahren für digitale Signaturen - wie eine fälschungssichere handschriftliche Unterschrift.

**Technisch:** Elliptic-curve signature scheme using Curve25519. Key size: 32 bytes (256 bits), signature: 64 bytes.

**Vorteile:**
- **Schnell:** Signierung und Verifikation
- **Klein:** Kleine Schlüssel und Signaturen
- **Sicher:** Hohe Sicherheit bei kleiner Schlüssellänge

**Im Projekt:** Standard für:
- Manifest-Signaturen
- Registry-Entry-Signaturen
- Key-Attestierungen

**Code-Beispiel:**
```rust
let secret_key = SigningKey::from_bytes(&secret_key_bytes);
let signature = secret_key.sign(message);
```

**Verwandte Begriffe:** [Digital Signature](#digital-signature), [Public Key](#public-key), [Private Key](#private-key)

---

### Merkle Tree (Merkle-Baum) ⭐
**Einfach:** Eine Struktur, die viele Daten-Elemente effizient zu einem einzelnen "Siegel" zusammenfasst.

**Technisch:** Tree structure where each leaf node is a hash of data, and each parent node is a hash of its children, culminating in a single root hash.

**Analogie:** Wie ein Baumdiagramm, wo jedes Blatt ein Dokument ist und der Stamm das Gesamt-Siegel aller Dokumente.

**Struktur:**
```
         Root (0xabcd...)
           /        \
    0x1234...    0x5678...
     /    \        /    \
  Leaf1 Leaf2  Leaf3 Leaf4
  (Doc1)(Doc2) (Doc3)(Doc4)
```

**Vorteil:** Man kann beweisen, dass ein bestimmtes Blatt im Baum ist, ohne alle anderen Blätter zu zeigen (Merkle Proof).

**Im Projekt:** Verwendet für Supplier-Root, UBO-Root, Company-Commitment-Root.

**Verwandte Begriffe:** [BLAKE3](#blake3), [Commitment](#commitment), [Root Hash](#root-hash)

---

### Zero-Knowledge Proof (ZKP) ⭐
**Einfach:** Ein Beweis, dass eine Aussage wahr ist, OHNE die zugrunde liegenden Daten preiszugeben.

**Technisch:** Cryptographic protocol where one party (prover) proves to another (verifier) that a statement is true without revealing any information beyond validity.

**Analogie:** Wie ein Altersnachweis, der nur "über 18" zeigt, nicht aber das Geburtsdatum oder Namen.

**Beispiel:**
- **Mit ZKP:** "Ich beweise, dass ich > 100 Lieferanten habe" → Prüfer sieht nur ✅
- **Ohne ZKP:** "Hier ist die Liste meiner 150 Lieferanten" → Prüfer sieht alles

**Im Projekt:** Geplant für Phase 4 (aktuell: Mock-Proofs).

**Verwandte Begriffe:** [Proof](#proof), [Statement](#statement), [Verifier](#verifier)

---

### Commitment
**Einfach:** Eine kryptographische "Verpflichtung" auf Daten, ohne die Daten selbst preiszugeben.

**Technisch:** Cryptographic binding to data that reveals nothing about the data until "opening" phase, while preventing later changes.

**Analogie:** Wie ein versiegelter Briefumschlag mit einem Tipp - man weiß, dass etwas drin ist, aber nicht was, bis man ihn öffnet.

**Im Projekt:** Merkle-Roots dienen als Commitments für Supplier/UBO-Daten.

**Datenstruktur:**
```rust
struct Commitments {
    supplier_root: String,              // 0x + 64 hex
    ubo_root: String,
    company_commitment_root: String,
}
```

**Verwandte Begriffe:** [Merkle Tree](#merkle-tree), [Root Hash](#root-hash)

---

### Signature (Digitale Signatur) ⭐
**Einfach:** Eine digitale "Unterschrift", die beweist, wer ein Dokument erstellt hat und dass es nicht verändert wurde.

**Technisch:** Cryptographic value derived from data using private key, verifiable with corresponding public key.

**Eigenschaften:**
- **Authentizität:** Beweist Identität des Absenders
- **Integrität:** Beweist, dass Daten nicht verändert wurden
- **Nicht-Abstreitbarkeit:** Absender kann nicht leugnen

**Im Projekt:** Ed25519-Signaturen für Manifests und Registry-Entries.

**Format:** Base64-codiert oder Hex (64 Bytes)

**Verwandte Begriffe:** [Ed25519](#ed25519), [Public Key](#public-key), [Private Key](#private-key)

---

### Public Key (Öffentlicher Schlüssel)
**Einfach:** Der "öffentliche" Teil eines Schlüsselpaars - kann verteilt werden, um Signaturen zu prüfen.

**Technisch:** Publicly shareable cryptographic key used to verify signatures created with corresponding private key.

**Analogie:** Wie eine Kontonummer - kann man jedem geben, damit sie Geld überweisen können (aber nicht abheben).

**Im Projekt:** 32 Bytes (Ed25519), gespeichert als Base64 in Key-Metadaten.

**Verwandte Begriffe:** [Private Key](#private-key), [Ed25519](#ed25519), [KID](#kid-key-id)

---

### Private Key (Privater Schlüssel)
**Einfach:** Der "geheime" Teil eines Schlüsselpaars - muss vertraulich bleiben, zum Signieren verwendet.

**Technisch:** Secret cryptographic key used to create digital signatures. Must never be shared or exposed.

**Analogie:** Wie eine PIN - nur Sie kennen sie, damit können Sie Dinge autorisieren.

**Im Projekt:** 32 Bytes (Ed25519), gespeichert in `.ed25519`-Dateien mit restriktiven Berechtigungen (chmod 600).

**Sicherheit:** ⚠️ Niemals in Git einchecken, niemals unverschlüsselt übertragen!

**Verwandte Begriffe:** [Public Key](#public-key), [Ed25519](#ed25519), [Key Management](#key-management)

---

### KID (Key ID) ⭐
**Einfach:** Eine eindeutige Kennung für einen Schlüssel (wie eine Seriennummer).

**Technisch:** 32-character hex string derived from BLAKE3 hash of base64-encoded public key (first 128 bits).

**Ableitung:**
```rust
kid = hex(BLAKE3(base64(public_key))[0:16])
```

**Beispiel:** `a1b2c3d4e5f67890a1b2c3d4e5f67890`

**Vorteil:** Deterministisch - gleicher Public Key erzeugt immer gleichen KID.

**Im Projekt:** Eingeführt in v0.10 für Key-Rotation und Chain-of-Trust.

**Verwandte Begriffe:** [Key Management](#key-management), [Public Key](#public-key), [Attestation](#attestation)

---

### Encryption (Verschlüsselung)
**Einfach:** Daten unleserlich machen, sodass nur berechtigte Personen sie wieder lesen können.

**Technisch:** Transformation of plaintext to ciphertext using cryptographic algorithms and keys.

**Arten:**
- **Symmetrisch:** Gleicher Schlüssel für Ver- und Entschlüsselung (z.B. AES)
- **Asymmetrisch:** Verschiedene Schlüssel (Public/Private Key)

**Analogie:** Wie ein Tresor mit Schloss - nur wer den Schlüssel hat, kommt rein.

**Im Projekt:** Wird aktuell nicht für Daten-Verschlüsselung verwendet (stattdessen: Hashes), aber für TLS-Verbindungen.

**Verwandte Begriffe:** [TLS](#tls-transport-layer-security), [Cipher](#cipher)

---

### Salt
**Einfach:** Zufällige Daten, die zu Passwörtern hinzugefügt werden, bevor sie gehasht werden.

**Technisch:** Random data added to input before hashing to prevent rainbow table attacks and ensure unique hashes for identical inputs.

**Analogie:** Wie ein einzigartiges Gewürz für jedes Gericht - selbst gleiche Rezepte schmecken anders.

**Im Projekt:** Nicht explizit verwendet (keine Passwort-Hashes), aber relevant für Key Derivation.

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [Nonce](#nonce)

---

### Nonce
**Einfach:** Eine Zahl, die nur einmal verwendet wird (Number used ONCE) - verhindert Replay-Angriffe.

**Technisch:** Arbitrary number used only once in cryptographic communication to prevent replay attacks.

**Analogie:** Wie eine Losnummer bei einer Verlosung - jede Nummer ist einzigartig.

**Verwendung:** In Challenge-Response-Protokollen, Blockchain (Mining), Timestamps.

**Im Projekt:** Im Audit-Trail durch Timestamps impliziert.

**Verwandte Begriffe:** [Timestamp](#timestamp), [Replay Attack](#replay-attack)

---

### Rainbow Table
**Einfach:** Vorberechnete Tabelle von Passwort-Hashes zum Knacken von Passwörtern.

**Technisch:** Precomputed table of hash values for common passwords, used to reverse hash functions.

**Abwehr:** Verwendung von Salt macht Rainbow Tables nutzlos.

**Analogie:** Wie ein Wörterbuch für verschlüsselte Wörter - man schlägt einfach nach.

**Im Projekt:** Nicht relevant (keine Passwort-Authentifizierung), aber guter Grund für Salts.

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [Salt](#salt)

---

### Collision (Hash-Kollision)
**Einfach:** Wenn zwei verschiedene Eingaben denselben Hash erzeugen (extrem selten bei guten Hash-Funktionen).

**Technisch:** Event where two different inputs produce identical hash output.

**Wahrscheinlichkeit:**
- **SHA3-256:** ~2^-256 (praktisch unmöglich)
- **BLAKE3:** Ähnlich sicher

**Im Projekt:** Wird als unmöglich angenommen (kryptographische Sicherheit basiert darauf).

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [Birthday Paradox](#birthday-paradox)

---

### Side-Channel Attack
**Einfach:** Angriff, der nicht den Code knackt, sondern z.B. Stromverbrauch oder Zeitdauer misst.

**Technisch:** Attack based on information leaked from physical implementation (timing, power consumption, EM radiation).

**Beispiele:**
- **Timing Attack:** Messung wie lange eine Operation dauert
- **Power Analysis:** Messung des Stromverbrauchs

**Abwehr:** Constant-time Operationen, Isolation.

**Im Projekt:** Rust's `subtle` crate verwendet für constant-time Vergleiche.

**Verwandte Begriffe:** [Timing Attack](#timing-attack), [Constant Time](#constant-time)

---

### Entropy (Entropie)
**Einfach:** Maß für Zufälligkeit oder Unvorhersehbarkeit von Daten.

**Technisch:** Measure of randomness or unpredictability in information theory (bits of entropy).

**Analogie:** Wie die "Wildheit" eines Würfels - ein fairer Würfel hat höhere Entropie als einer mit Gewicht.

**Wichtig für:** Schlüsselerzeugung, Zufallszahlen, Sicherheit.

**Im Projekt:** `rand` crate für kryptographisch sichere Zufallszahlen.

**Verwandte Begriffe:** [CSPRNG](#csprng), [Random](#random)

---

### CSPRNG (Cryptographically Secure Pseudo-Random Number Generator)
**Einfach:** Ein Zufallszahlengenerator, der sicher genug für Kryptographie ist.

**Technisch:** PRNG that produces output indistinguishable from true randomness and resistant to prediction.

**Im Projekt:** `rand` crate verwendet OS-Entropie (z.B. `/dev/urandom`).

**Verwandte Begriffe:** [Entropy](#entropy-entropie), [Random](#random)

---

### Cipher
**Einfach:** Ein Verschlüsselungs-Algorithmus.

**Technisch:** Algorithm for performing encryption and decryption.

**Beispiele:**
- **AES:** Symmetrischer Cipher (Block Cipher)
- **ChaCha20:** Stream Cipher
- **RSA:** Asymmetrischer Cipher

**Im Projekt:** TLS-Cipher Suites (z.B. `TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384`).

**Verwandte Begriffe:** [Encryption](#encryption-verschlüsselung), [TLS](#tls-transport-layer-security)

---

## 3. Blockchain & Distributed Ledger

### Blockchain
**Einfach:** Eine Kette von "Blöcken" (Datengruppen), die manipulationssicher verkettet sind.

**Technisch:** Distributed, append-only ledger where each block contains cryptographic hash of previous block, timestamp, and transaction data.

**Analogie:** Wie ein Fahrtenbuch mit nummerierten, miteinander verketteten Seiten - kann man nicht einzelne Seiten entfernen ohne dass es auffällt.

**Im Projekt:** Audit-Trail verwendet ähnliches Konzept (Hash-Chain), aber zentral statt verteilt.

**Verwandte Begriffe:** [Hash Chain](#hash-chain), [Distributed Ledger](#distributed-ledger)

---

### Hash Chain ⭐
**Einfach:** Eine Kette von Ereignissen, wo jedes neue Ereignis den Hash des vorherigen enthält - manipulationssicher.

**Technisch:** Successive application of hash function where each output depends on all previous inputs.

**Struktur:**
```
Event 0: hash_0 = SHA3(0x0 || data_0)
Event 1: hash_1 = SHA3(hash_0 || data_1)
Event 2: hash_2 = SHA3(hash_1 || data_2)
```

**Im Projekt:** Audit-Trail implementiert als Hash-Chain mit SHA3-256.

**Vorteil:** Manipulation würde alle nachfolgenden Hashes ändern → sofort erkennbar.

**Verwandte Begriffe:** [Audit Trail](#audit-trail), [Tamper-Evident](#tamper-evident)

---

### Distributed Ledger
**Einfach:** Ein "Hauptbuch", das auf vielen Computern gleichzeitig gespeichert ist (dezentral).

**Technisch:** Database replicated across multiple nodes in network, ensuring consensus without central authority.

**Arten:**
- **Permissionless:** Jeder kann teilnehmen (z.B. Bitcoin)
- **Permissioned:** Nur autorisierte Teilnehmer (z.B. Hyperledger)

**Im Projekt:** Nicht verwendet (zentrale Architektur), aber als Option für Public Anchor (Phase 4).

**Verwandte Begriffe:** [Blockchain](#blockchain), [Consensus](#consensus)

---

### Consensus (Konsens)
**Einfach:** Einigung in einem dezentralen Netzwerk darüber, was die "Wahrheit" ist.

**Technisch:** Agreement protocol in distributed systems to ensure all nodes agree on shared state.

**Algorithmen:**
- **PoW (Proof of Work):** Bitcoin
- **PoS (Proof of Stake):** Ethereum 2.0
- **PBFT:** Byzantine Fault Tolerance

**Im Projekt:** Nicht relevant (zentrale Architektur).

**Verwandte Begriffe:** [Distributed Ledger](#distributed-ledger), [Byzantine Fault](#byzantine-fault)

---

### Smart Contract
**Einfach:** Programm, das auf einer Blockchain läuft und automatisch ausgeführt wird (z.B. "Wenn X, dann Y").

**Technisch:** Self-executing code deployed on blockchain that runs when predetermined conditions are met.

**Beispiel:** "Wenn Lieferung bestätigt, überweise Zahlung automatisch."

**Im Projekt:** Nicht verwendet, aber relevant für Public Anchor (Ethereum, Hedera).

**Verwandte Begriffe:** [Blockchain](#blockchain), [Solidity](#solidity)

---

### Timestamping (Zeitstempel)
**Einfach:** Beweis, dass ein Dokument zu einem bestimmten Zeitpunkt existiert hat.

**Technisch:** Cryptographic binding of data to specific time, often using trusted third party (TSA).

**Arten:**
- **RFC3339:** Standardformat für Zeitstempel (ISO 8601)
- **Unix Timestamp:** Sekunden seit 1970-01-01

**Im Projekt:** Alle Events haben RFC3339-Timestamps. Optional: Dual-Anchor mit TSA.

**Verwandte Begriffe:** [TSA](#tsa-time-stamp-authority), [RFC3339](#rfc3339)

---

## 4. Software-Architektur

### Layered Architecture (Schichtenarchitektur) ⭐
**Einfach:** Aufbau einer Software in "Stockwerken" mit klaren Verantwortlichkeiten.

**Technisch:** Architectural pattern organizing system into hierarchical layers with defined responsibilities and interfaces.

**Schichten im Projekt:**
1. **Presentation Layer:** CLI, REST API
2. **Business Logic Layer:** Proof Engine, Verifier
3. **Core Services Layer:** Crypto, Audit
4. **Data Layer:** SQLite, JSON

**Vorteil:** Austauschbar, wartbar, testbar.

**Analogie:** Wie ein Gebäude - jedes Stockwerk hat eine Aufgabe, aber sie bauen aufeinander auf.

**Verwandte Begriffe:** [Separation of Concerns](#separation-of-concerns), [MVC](#mvc)

---

### Microservices
**Einfach:** Aufteilung einer Anwendung in viele kleine, unabhängige Dienste.

**Technisch:** Architectural style structuring application as collection of loosely coupled, independently deployable services.

**Im Projekt:** Nicht verwendet (monolithisch), aber API-Server ist service-ready.

**Vorteil:** Skalierbarkeit, Technologie-Unabhängigkeit.
**Nachteil:** Komplexität, Netzwerk-Overhead.

**Verwandte Begriffe:** [Monolith](#monolith), [Service-Oriented Architecture](#soa)

---

### Monolith
**Einfach:** Alle Funktionen einer Anwendung sind in einem einzigen Programm.

**Technisch:** Application architecture where all components are tightly coupled in single codebase and deployment unit.

**Im Projekt:** Aktueller Ansatz - ein Binary mit allen Features.

**Vorteil:** Einfach zu entwickeln und deployen.
**Nachteil:** Schwieriger zu skalieren.

**Verwandte Begriffe:** [Microservices](#microservices), [Binary](#binary)

---

### Separation of Concerns ⭐
**Einfach:** Jeder Teil des Programms kümmert sich nur um eine Sache.

**Technisch:** Design principle separating program into distinct sections, each addressing separate concern.

**Beispiel im Projekt:**
- `crypto/` nur Kryptographie
- `verifier/` nur Verifikation
- `api/` nur HTTP-Handling

**Vorteil:** Wartbarkeit, Testbarkeit, Wiederverwendbarkeit.

**Verwandte Begriffe:** [Layered Architecture](#layered-architecture-schichtenarchitektur), [Module](#module-rust)

---

### Trait (Rust) ⭐
**Einfach:** Eine "Schnittstelle" in Rust - definiert, welche Funktionen ein Typ haben muss.

**Technisch:** Rust's mechanism for defining shared behavior through interface-like abstractions.

**Beispiel im Projekt:**
```rust
trait ProofSystem {
    fn backend_name(&self) -> &str;
    fn verify(&self, proof_data: &ProofData) -> Result<bool>;
}
```

**Analogie:** Wie ein "Vertrag" - jeder, der diesen Vertrag unterschreibt, muss bestimmte Funktionen erfüllen.

**Verwandte Begriffe:** [Interface](#interface), [Polymorphism](#polymorphism)

---

### Factory Pattern
**Einfach:** Ein Muster, das Objekte erstellt, ohne den genauen Typ zu kennen.

**Technisch:** Creational design pattern providing interface for creating objects without specifying exact classes.

**Im Projekt:**
```rust
fn backend_factory(backend: &str) -> Box<dyn ProofSystem> {
    match backend {
        "mock" => Box::new(MockZK),
        "halo2" => Box::new(Halo2ZK),
        _ => panic!("Unknown backend"),
    }
}
```

**Vorteil:** Flexibilität, Erweiterbarkeit.

**Verwandte Begriffe:** [Design Pattern](#design-pattern), [Abstract Factory](#abstract-factory)

---

### Dependency Injection
**Einfach:** Abhängigkeiten werden von außen "hereingereicht" statt intern erstellt.

**Technisch:** Design pattern where dependencies are provided to object from external source.

**Beispiel:**
```rust
// Statt:
let db = Database::new();
let service = Service::new(db);

// Besser:
fn new_service(db: Database) -> Service { ... }
```

**Vorteil:** Testbarkeit (Mock-Objekte), Flexibilität.

**Verwandte Begriffe:** [Inversion of Control](#ioc), [Testing](#testing--quality-assurance)

---

### Orchestrator ⭐
**Einfach:** Ein "Dirigent", der verschiedene Komponenten koordiniert.

**Technisch:** Component responsible for coordinating execution flow across multiple services or modules.

**Im Projekt:** `orchestrator/` Module koordiniert Proof-Backend-Auswahl und Execution Planning.

**Funktionen:**
- Backend-Selektion (risk-based)
- Execution Planning
- Policy Enforcement

**Verwandte Begriffe:** [Coordinator](#coordinator), [Service Mesh](#service-mesh)

---

### Middleware ⭐
**Einfach:** Software, die "in der Mitte" sitzt und Anfragen bearbeitet, bevor sie zum Hauptprogramm kommen.

**Technisch:** Software layer between request and application logic, processing requests/responses in pipeline.

**Im Projekt:** Axum Middleware für:
- Authentifizierung (OAuth2)
- Logging (tracing)
- Metrics (Prometheus)
- CORS

**Analogie:** Wie ein Filter oder Sieb - alles muss durchlaufen, bevor es ankommt.

**Verwandte Begriffe:** [Axum](#axum), [Tower](#tower)

---

### Idempotent
**Einfach:** Eine Operation, die man mehrfach ausführen kann, ohne dass sich das Ergebnis ändert.

**Technisch:** Operation that produces same result regardless of how many times it's executed.

**Beispiele:**
- **Idempotent:** `SET x = 5` (egal wie oft, x ist 5)
- **Nicht idempotent:** `x = x + 1` (jedes Mal anders)

**Im Projekt:** REST API sollte idempotent sein (GET, PUT, DELETE ja; POST nein).

**Verwandte Begriffe:** [REST](#rest-representational-state-transfer), [HTTP Methods](#http-methods)

---

### Stateless ⭐
**Einfach:** Der Server merkt sich nichts zwischen Anfragen - jede Anfrage ist unabhängig.

**Technisch:** Architecture where server doesn't maintain session state between requests.

**Vorteil:** Einfach skalierbar (Load Balancing), kein Session-Speicher nötig.

**Im Projekt:** REST API ist stateless (außer Registry/BLOB Store).

**Analogie:** Wie ein Münztelefon - jedes Mal muss man neu bezahlen, es merkt sich nichts.

**Verwandte Begriffe:** [REST](#rest-representational-state-transfer), [Horizontal Scaling](#horizontal-scaling)

---

### ACID
**Einfach:** Eigenschaften, die eine sichere Datenbank ausmachen: Atomicity, Consistency, Isolation, Durability.

**Technisch:** Set of properties guaranteeing reliable database transactions.

**Bedeutung:**
- **Atomicity:** Ganz oder gar nicht
- **Consistency:** Datenbank bleibt konsistent
- **Isolation:** Transaktionen beeinflussen sich nicht
- **Durability:** Committed = dauerhaft gespeichert

**Im Projekt:** SQLite garantiert ACID-Eigenschaften.

**Verwandte Begriffe:** [Transaction](#transaction), [SQLite](#sqlite)

---

### Race Condition
**Einfach:** Fehler, der auftritt, wenn zwei Prozesse gleichzeitig auf dieselben Daten zugreifen.

**Technisch:** Flaw where system behavior depends on timing of uncontrollable events.

**Beispiel:** Zwei Prozesse lesen x=10, beide erhöhen um 1, beide schreiben 11 statt 12.

**Abwehr:** Locks, Mutexes, Transaktionen.

**Im Projekt:** SQLite WAL-Mode minimiert Locks.

**Verwandte Begriffe:** [Mutex](#mutex), [Lock](#lock)

---

### Pipeline
**Einfach:** Eine Abfolge von Verarbeitungsschritten, wo die Ausgabe eines Schritts die Eingabe des nächsten ist.

**Technisch:** Series of processing stages where output of one stage is input of next.

**Im Projekt:** Core Processing Pipeline:
```
CSV → Commitment → Manifest → Proof → Verification
```

**Analogie:** Wie ein Fließband in einer Fabrik.

**Verwandte Begriffe:** [Data Flow](#data-flow), [ETL](#etl)

---

### Plugin Architecture
**Einfach:** System, das erweiterbar ist durch externe Module ("Plugins").

**Technisch:** Architectural pattern enabling extension through dynamically loaded modules.

**Im Projekt:** Proof-Backends als Plugins (Mock, Halo2, Spartan) via Trait-Implementierung.

**Vorteil:** Erweiterbar ohne Kern-Code zu ändern.

**Verwandte Begriffe:** [Trait](#trait-rust), [WASM](#wasm--webassembly)

---

## 5. APIs & Schnittstellen

### API (Application Programming Interface) ⭐
**Einfach:** Eine "Steckdose für Software" - definierte Art, wie Programme miteinander kommunizieren.

**Technisch:** Set of definitions, protocols, and tools for building software applications, specifying how components interact.

**Analogie:** Wie USB-Anschlüsse - standardisierte Verbindung, in die verschiedene Geräte passen.

**Arten:**
- **REST API:** Über HTTP
- **GraphQL:** Query-basiert
- **gRPC:** Binär-Protokoll

**Im Projekt:** REST API mit JSON für Integration mit anderen Systemen (z.B. SAP).

**Verwandte Begriffe:** [REST](#rest-representational-state-transfer), [Endpoint](#endpoint)

---

### REST (Representational State Transfer) ⭐
**Einfach:** Ein Standard für Web-APIs - verwendet HTTP-Methoden (GET, POST, PUT, DELETE).

**Technisch:** Architectural style for distributed hypermedia systems, using stateless client-server communication over HTTP.

**Prinzipien:**
- **Stateless:** Keine Session-Verwaltung
- **Cacheable:** Responses können gecacht werden
- **Uniform Interface:** Standardisierte Methoden

**Im Projekt:** REST API v0.11.0 mit Axum Framework.

**HTTP-Methoden:**
- `GET` - Lesen
- `POST` - Erstellen
- `PUT` - Aktualisieren
- `DELETE` - Löschen

**Verwandte Begriffe:** [API](#api-application-programming-interface), [HTTP](#http-hypertext-transfer-protocol)

---

### Endpoint ⭐
**Einfach:** Eine spezifische URL in einer API, die eine bestimmte Funktion ausführt.

**Technisch:** Specific URL path in API that handles particular request type.

**Beispiele im Projekt:**
- `GET /healthz` - Health Check
- `POST /verify` - Proof Verification
- `GET /metrics` - Prometheus Metrics

**Format:** `[HTTP-Methode] [Pfad]`

**Verwandte Begriffe:** [REST](#rest-representational-state-transfer), [Route](#route)

---

### HTTP (Hypertext Transfer Protocol)
**Einfach:** Das Protokoll, mit dem Browser und Server kommunizieren.

**Technisch:** Application-layer protocol for transmitting hypermedia documents (HTML, JSON, etc.).

**Versionen:**
- **HTTP/1.1:** Text-basiert (Standard)
- **HTTP/2:** Binär, Multiplexing
- **HTTP/3:** QUIC-basiert

**Im Projekt:** HTTP/1.1 und HTTP/2 unterstützt (via Axum).

**Verwandte Begriffe:** [REST](#rest-representational-state-transfer), [HTTPS](#https)

---

### HTTPS (HTTP Secure)
**Einfach:** HTTP mit Verschlüsselung (via TLS) - die sichere Variante von HTTP.

**Technisch:** HTTP over TLS/SSL, providing encrypted communication and authentication.

**Im Projekt:** Standard für Produktion (Port 8443), mit rustls implementiert.

**Erkennbar an:** `https://` und Schloss-Symbol im Browser.

**Verwandte Begriffe:** [TLS](#tls-transport-layer-security), [Certificate](#certificate)

---

### JSON (JavaScript Object Notation) ⭐
**Einfach:** Ein Format zum Speichern strukturierter Daten in Textform (wie XML, aber einfacher).

**Technisch:** Lightweight data-interchange format, human-readable, language-independent.

**Beispiel:**
```json
{
  "name": "Max Mustermann",
  "alter": 35,
  "aktiv": true
}
```

**Im Projekt:** Standard für:
- API Requests/Responses
- Manifest-Dateien
- Registry-Einträge (JSON-Backend)

**Verwandte Begriffe:** [YAML](#yaml), [Serialization](#serialization)

---

### YAML (YAML Ain't Markup Language)
**Einfach:** Ein Format für Konfigurationsdateien - menschenlesbarer als JSON.

**Technisch:** Human-readable data serialization standard, superset of JSON with additional features.

**Beispiel:**
```yaml
name: Max Mustermann
alter: 35
aktiv: true
hobbies:
  - Lesen
  - Sport
```

**Im Projekt:** Verwendet für Policy-Definitionen (`policy.lksg.v1.yml`).

**Vorteil:** Kommentare möglich, weniger Klammern.

**Verwandte Begriffe:** [JSON](#json-javascript-object-notation), [TOML](#toml)

---

### CLI (Command Line Interface) ⭐
**Einfach:** Bedienung über Textbefehle in einem Terminal (keine grafische Oberfläche).

**Technisch:** Text-based user interface for interacting with programs through typed commands.

**Beispiel:**
```bash
cap prepare --suppliers suppliers.csv --ubos ubos.csv
```

**Vorteil:**
- Automatisierbar (Skripte)
- Ressourcenschonend
- Remote-Zugriff einfach

**Im Projekt:** `main.rs` implementiert CLI mit `clap` crate.

**Verwandte Begriffe:** [Terminal](#terminal), [Shell](#shell)

---

### Query Parameter
**Einfach:** Zusätzliche Informationen in einer URL (nach `?`).

**Technisch:** Key-value pairs appended to URL for passing data to server.

**Beispiel:**
```
/registry/find?manifest_hash=0xabcd&proof_hash=0x1234
                ^--- Query Parameter
```

**Im Projekt:** Verwendet in Registry-API für Suchparameter.

**Verwandte Begriffe:** [URL](#url), [Path Parameter](#path-parameter)

---

### Path Parameter
**Einfach:** Variable Teile einer URL (z.B. eine ID).

**Technisch:** Dynamic segments in URL path representing resource identifiers.

**Beispiel:**
```
/policy/0xabcd1234
         ^--- Path Parameter (Policy ID)
```

**Im Projekt:** `GET /policy/:id` - `:id` ist Path Parameter.

**Verwandte Begriffe:** [URL](#url), [Query Parameter](#query-parameter)

---

### Status Code (HTTP) ⭐
**Einfach:** Dreistellige Zahl, die das Ergebnis einer HTTP-Anfrage anzeigt.

**Technisch:** Three-digit integer indicating outcome of HTTP request.

**Kategorien:**
- **2xx:** Erfolg (z.B. 200 OK)
- **3xx:** Umleitung
- **4xx:** Client-Fehler (z.B. 404 Not Found)
- **5xx:** Server-Fehler (z.B. 500 Internal Server Error)

**Wichtigste im Projekt:**
- `200 OK` - Erfolg
- `400 Bad Request` - Ungültige Anfrage
- `401 Unauthorized` - Fehlende/ungültige Authentifizierung
- `403 Forbidden` - Zugriff verweigert
- `404 Not Found` - Ressource nicht gefunden
- `500 Internal Server Error` - Server-Fehler

**Verwandte Begriffe:** [HTTP](#http-hypertext-transfer-protocol), [REST](#rest-representational-state-transfer)

---

### Content-Type
**Einfach:** HTTP-Header, der angibt, welches Format die Daten haben.

**Technisch:** HTTP header indicating media type of resource or data being sent/received.

**Beispiele:**
- `application/json` - JSON-Daten
- `text/html` - HTML-Seite
- `application/octet-stream` - Binärdaten

**Im Projekt:** Standard ist `application/json` für API-Requests/Responses.

**Verwandte Begriffe:** [HTTP Header](#http-header), [MIME Type](#mime-type)

---

## 6. Datenformate & Serialisierung

### Serialization
**Einfach:** Umwandlung von Datenstrukturen in ein speicherbares/übertragbares Format.

**Technisch:** Process of converting data structures into byte stream or text format for storage or transmission.

**Gegenteil:** Deserialization (Rückwandlung)

**Formate:**
- JSON
- YAML
- Binary (z.B. CAPZ)

**Im Projekt:** `serde` crate für (De-)Serialisierung.

**Verwandte Begriffe:** [JSON](#json-javascript-object-notation), [serde](#serde-rust)

---

### CSV (Comma-Separated Values)
**Einfach:** Tabellenformat, wo Spalten durch Kommas getrennt sind (wie Excel, aber simpler).

**Technisch:** Plain text format for tabular data with fields separated by delimiters (typically commas).

**Beispiel:**
```csv
name,jurisdiction,tier
"Supplier A",DE,TIER_1
"Supplier B",FR,TIER_2
```

**Im Projekt:** Input-Format für Supplier- und UBO-Daten.

**Vorteil:** Einfach, mit Excel bearbeitbar.

**Verwandte Begriffe:** [TSV](#tsv), [Excel](#excel)

---

### Base64
**Einfach:** Kodierung von Binärdaten in Text (nur Buchstaben und Zahlen).

**Technisch:** Encoding scheme converting binary data to ASCII text using 64-character set.

**Verwendung:** E-Mail-Anhänge, URLs, JSON-Einbettung von Binärdaten.

**Im Projekt:**
- Ed25519 Public Keys
- Signaturen
- Proof Data

**Beispiel:**
```
Binär:  [0x41, 0x42, 0x43]
Base64: "QUJD"
```

**Verwandte Begriffe:** [Encoding](#encoding), [Hex](#hexadecimal)

---

### Hexadecimal (Hex)
**Einfach:** Zahlensystem mit 16 Ziffern (0-9, A-F) statt 10.

**Technisch:** Base-16 numeral system using digits 0-9 and A-F.

**Verwendung:** Hashes, Speicheradressen, Farben.

**Im Projekt:** Hash-Darstellung mit `0x`-Präfix:
```
0x83a8779ddef4567890...
```

**Umrechnung:**
- 1 Byte (8 Bit) = 2 Hex-Zeichen
- 32 Bytes = 64 Hex-Zeichen

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [Base64](#base64)

---

### RFC3339
**Einfach:** Standard-Format für Datum und Uhrzeit (ISO 8601).

**Technisch:** Internet timestamp format (subset of ISO 8601) with timezone information.

**Format:** `YYYY-MM-DDThh:mm:ssZ`

**Beispiel:**
```
2025-11-17T10:30:00Z
```

**Bedeutung:**
- `T` trennt Datum und Zeit
- `Z` bedeutet UTC (Zulu Time)
- Alternative: `+01:00` für Timezone-Offset

**Im Projekt:** Standard für alle Timestamps (created_at, timestamp, etc.).

**Verwandte Begriffe:** [ISO 8601](#iso-8601), [Timestamp](#timestamp)

---

### UUID (Universally Unique Identifier)
**Einfach:** Eindeutige Kennung, die garantiert weltweit einzigartig ist.

**Technisch:** 128-bit identifier designed to be globally unique without central coordination.

**Format:** `550e8400-e29b-41d4-a716-446655440000`

**Im Projekt:** Registry-Entry IDs.

**Versionen:**
- **v4:** Zufällig (am häufigsten)
- **v7:** Zeitbasiert (sortierbar)

**Verwandte Begriffe:** [GUID](#guid), [ID](#identifier)

---

### Schema
**Einfach:** Eine Beschreibung der Struktur von Daten (welche Felder, welche Typen).

**Technisch:** Formal description of data structure defining fields, types, and constraints.

**Im Projekt:**
- JSON Schema für Manifest (`docs/manifest.schema.json`)
- SQLite Schema für Registry (`registry/schema.rs`)

**Beispiel (JSON Schema):**
```json
{
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "age": {"type": "integer"}
  },
  "required": ["name"]
}
```

**Verwandte Begriffe:** [Validation](#validation), [Type System](#type-system)

---

### Encoding
**Einfach:** Umwandlung von Daten in ein bestimmtes Format (z.B. Text → UTF-8 Bytes).

**Technisch:** Process of converting data from one format to another using predefined rules.

**Arten:**
- **Character Encoding:** UTF-8, ASCII, Latin-1
- **Binary Encoding:** Base64, Hex
- **Compression:** gzip, zstd

**Im Projekt:**
- UTF-8 für alle Text-Dateien
- Base64 für Binärdaten in JSON
- Hex für Hashes

**Verwandte Begriffe:** [UTF-8](#utf-8), [Base64](#base64)

---

## 7. Datenbank & Storage

### SQLite ⭐
**Einfach:** Eine kleine, eingebettete Datenbank (kein separater Server nötig).

**Technisch:** Self-contained, serverless, zero-configuration SQL database engine stored as single file.

**Vorteile:**
- Keine Installation nötig
- ACID-konform
- Schnell für < 100GB Daten

**Im Projekt:** Registry-Backend Option (performanter als JSON).

**Datei:** `registry.db`

**Verwandte Begriffe:** [SQL](#sql), [ACID](#acid), [WAL](#wal-write-ahead-logging)

---

### SQL (Structured Query Language)
**Einfach:** Standardisierte Sprache zum Abfragen und Verwalten von Datenbanken.

**Technisch:** Domain-specific language for managing relational databases.

**Basis-Befehle:**
```sql
SELECT * FROM registry_entries WHERE status = 'ok';
INSERT INTO registry_entries (id, manifest_hash) VALUES ('abc', '0x123');
UPDATE registry_entries SET status = 'verified' WHERE id = 'abc';
DELETE FROM registry_entries WHERE id = 'abc';
```

**Im Projekt:** SQLite-Backend nutzt SQL für Registry-Operationen.

**Verwandte Begriffe:** [SQLite](#sqlite), [Query](#query)

---

### WAL (Write-Ahead Logging) ⭐
**Einfach:** Eine Technik, die Schreibvorgänge in einer Datenbank beschleunigt.

**Technisch:** Database optimization where changes are first written to log file, then applied to database asynchronously.

**Vorteil:**
- Schnellere Writes
- Gleichzeitige Reads während Writes
- Crash-Recovery

**Im Projekt:** SQLite mit WAL-Mode (`PRAGMA journal_mode=WAL`).

**Dateien:** `registry.db`, `registry.db-wal`, `registry.db-shm`

**Verwandte Begriffe:** [SQLite](#sqlite), [Transaction](#transaction)

---

### Index (Datenbank)
**Einfach:** Wie ein Stichwortverzeichnis in einem Buch - macht Suche viel schneller.

**Technisch:** Data structure improving speed of data retrieval operations on database tables.

**Im Projekt:** SQLite-Indexes:
```sql
CREATE INDEX idx_manifest_proof ON registry_entries(manifest_hash, proof_hash);
CREATE INDEX idx_timestamp ON registry_entries(timestamp);
CREATE INDEX idx_kid ON registry_entries(kid);
```

**Vorteil:** 100x schnellere Suche (in Tests: 428 µs → 9.5 µs).

**Nachteil:** Verlangsamt Writes, braucht Speicher.

**Verwandte Begriffe:** [SQLite](#sqlite), [Query Performance](#query-performance)

---

### BLOB (Binary Large Object)
**Einfach:** Große Binärdaten (z.B. Bilder, Videos, Dateien) in einer Datenbank.

**Technisch:** Collection of binary data stored as single entity in database.

**Im Projekt:** BLOB Store für Proof-Daten (nicht in DB, sondern als Dateien mit Refcounting).

**Verwandte Begriffe:** [Binary](#binary), [Storage](#storage)

---

### Content-Addressable Storage ⭐
**Einfach:** Speicher-System, wo Dateien durch ihren Hash-Wert identifiziert werden (nicht durch Namen).

**Technisch:** Storage system using content hash as address/identifier, enabling automatic deduplication.

**Vorteil:**
- **Deduplizierung:** Gleiche Daten werden nur einmal gespeichert
- **Integrität:** Hash beweist Unverfälschtheit

**Im Projekt:** BLOB Store implementiert Content-Addressable Storage mit BLAKE3.

**Beispiel:**
```
Hash: 0x83a877...
→ Datei: blobs/83/a877...
```

**Verwandte Begriffe:** [Hash](#hash--hash-funktion), [BLOB Store](#blob-binary-large-object)

---

### Deduplication
**Einfach:** Automatisches Erkennen und Entfernen von Duplikaten.

**Technisch:** Process of eliminating duplicate copies of data.

**Im Projekt:** BLOB Store dedupliziert automatisch - gleicher Content = gleicher Hash = nur einmal gespeichert.

**Verwandte Begriffe:** [Content-Addressable Storage](#content-addressable-storage), [Hash](#hash--hash-funktion)

---

### Refcount (Reference Count)
**Einfach:** Zähler, wie oft auf etwas verwiesen wird - wird bei 0 gelöscht.

**Technisch:** Count of references to object; when reaches zero, object can be garbage collected.

**Im Projekt:** BLOB Store zählt Referenzen auf BLOBs.

**Beispiel:**
- BLOB mit Hash `0xabc` wird von 3 Manifests verwendet → refcount = 3
- Manifest wird gelöscht → refcount = 2
- Bei refcount = 0 → BLOB kann gelöscht werden (GC)

**Verwandte Begriffe:** [Garbage Collection](#garbage-collection-gc), [Memory Management](#memory-management)

---

### Garbage Collection (GC)
**Einfach:** Automatisches Aufräumen von nicht mehr benötigtem Speicher.

**Technisch:** Automatic memory management reclaiming memory occupied by objects no longer referenced.

**Im Projekt:** BLOB Store GC entfernt BLOBs mit refcount = 0.

**Befehl:** `cap blob-store gc`

**Verwandte Begriffe:** [Refcount](#refcount-reference-count), [Memory Management](#memory-management)

---

### Persistence
**Einfach:** Dauerhafte Speicherung von Daten (überleben System-Neustart).

**Technisch:** Data surviving process termination, stored in non-volatile storage.

**Im Projekt:**
- SQLite-Datenbank (registry.db)
- JSON-Dateien (manifests, proofs)
- BLOB Store (Dateisystem)

**Verwandte Begriffe:** [Durability](#durability), [ACID](#acid)

---

## 8. Deployment & Container

### Docker ⭐
**Einfach:** Software-"Versandkisten", die ein Programm mit allen Abhängigkeiten verpacken.

**Technisch:** Platform for developing, shipping, and running applications in isolated containers.

**Vorteil:**
- Läuft überall gleich ("works on my machine" gelöst)
- Isoliert (keine Konflikte mit anderen Programmen)
- Einfach zu deployen

**Im Projekt:** Multi-Stage Dockerfile für optimierte Container-Images.

**Befehle:**
```bash
docker build -t cap-agent:0.11.0 .
docker run -p 8443:8443 cap-agent:0.11.0
```

**Verwandte Begriffe:** [Container](#container), [Dockerfile](#dockerfile)

---

### Container
**Einfach:** Isolierte Laufzeitumgebung für eine Anwendung (wie eine virtuelle Maschine, aber leichter).

**Technisch:** Lightweight, standalone executable package including application code, runtime, libraries, and settings.

**Analogie:** Wie ein Wohncontainer - komplett ausgestattet, überall einsetzbar.

**Unterschied zu VM:**
- **Container:** Teilt Kernel mit Host (leichter)
- **VM:** Eigener Kernel (schwerer)

**Im Projekt:** Docker Container für API-Server.

**Verwandte Begriffe:** [Docker](#docker), [Image](#image-docker)

---

### Kubernetes (K8s) ⭐
**Einfach:** Verwaltungssystem für viele Container - automatisches Starten, Stoppen, Skalieren.

**Technisch:** Container orchestration platform automating deployment, scaling, and management of containerized applications.

**Funktionen:**
- **Auto-Scaling:** Mehr Container bei hoher Last
- **Self-Healing:** Neustart bei Absturz
- **Load Balancing:** Verteilung von Anfragen

**Im Projekt:** K8s-Manifests für Enterprise-Deployment (3+ Replicas, Ingress, PVC).

**Verwandte Begriffe:** [Docker](#docker), [Pod](#pod-kubernetes)

---

### Dockerfile
**Einfach:** Rezept zum Bau eines Docker-Containers.

**Technisch:** Text document containing instructions for building Docker image.

**Beispiel:**
```dockerfile
FROM rust:1.75
COPY . /app
RUN cargo build --release
CMD ["./target/release/cap-verifier-api"]
```

**Im Projekt:** Multi-Stage Dockerfile (Build + Runtime getrennt für kleinere Images).

**Verwandte Begriffe:** [Docker](#docker), [Image](#image-docker)

---

### Image (Docker)
**Einfach:** Eine "Blaupause" für Container - wie ein Template.

**Technisch:** Read-only template containing application code, libraries, and dependencies.

**Analogie:** Wie eine ISO-Datei für eine CD - kann viele Kopien davon erstellen.

**Im Projekt:** `cap-agent:0.11.0` Image gebaut aus Dockerfile.

**Verwandte Begriffe:** [Docker](#docker), [Container](#container)

---

### Registry (Docker)
**Einfach:** Online-Speicher für Docker-Images (wie GitHub für Code).

**Technisch:** Repository for storing and distributing Docker images.

**Beispiele:**
- **Docker Hub:** Öffentlich
- **Private Registry:** Firmenintern

**Im Projekt:** Könnte Images in Private Registry pushen für interne Verteilung.

**Verwandte Begriffe:** [Docker](#docker), [Image](#image-docker)

---

### Pod (Kubernetes)
**Einfach:** Kleinste Einheit in Kubernetes - kann einen oder mehrere Container enthalten.

**Technisch:** Smallest deployable unit in K8s, encapsulating one or more containers sharing network and storage.

**Im Projekt:** Jeder Pod enthält einen `cap-verifier` Container.

**Verwandte Begriffe:** [Kubernetes](#kubernetes-k8s), [Container](#container)

---

### Service (Kubernetes)
**Einfach:** Netzwerk-Endpunkt für Pods - gibt Pods eine stabile Adresse.

**Technisch:** Abstraction defining logical set of Pods and policy for accessing them.

**Typen:**
- **ClusterIP:** Nur intern erreichbar
- **NodePort:** Über Node-IP + Port
- **LoadBalancer:** Externe IP (Cloud)

**Im Projekt:** `cap-verifier-svc` als ClusterIP-Service.

**Verwandte Begriffe:** [Kubernetes](#kubernetes-k8s), [Pod](#pod-kubernetes)

---

### Ingress (Kubernetes)
**Einfach:** "Eingang" in den Kubernetes-Cluster - verwaltet externen Zugriff (HTTPS, Domains).

**Technisch:** API object managing external access to services, typically HTTP/HTTPS with routing rules.

**Im Projekt:** Ingress mit cert-manager für Let's Encrypt TLS-Zertifikate.

**Funktionen:**
- SSL/TLS Termination
- Name-based Virtual Hosting
- Load Balancing

**Verwandte Begriffe:** [Kubernetes](#kubernetes-k8s), [Load Balancer](#load-balancer)

---

### PersistentVolume (PV)
**Einfach:** Dauerhafter Speicher in Kubernetes (überlebt Pod-Neustarts).

**Technisch:** Piece of storage in cluster provisioned by administrator or dynamically.

**Im Projekt:** PVC (PersistentVolumeClaim) für Registry-Datenbank und BLOB Store.

**Verwandte Begriffe:** [Kubernetes](#kubernetes-k8s), [Storage](#storage)

---

### Horizontal Scaling ⭐
**Einfach:** Mehr Server/Container hinzufügen statt einen stärkeren (scale out statt scale up).

**Technisch:** Increasing capacity by adding more instances rather than increasing power of single instance.

**Beispiel:**
- **Horizontal:** 10 kleine Server
- **Vertikal:** 1 großer Server

**Vorteil:** Bessere Ausfallsicherheit, einfacher skalierbar.

**Im Projekt:** API-Server ist stateless → einfach horizontal skalierbar (z.B. 3 Replicas in K8s).

**Verwandte Begriffe:** [Kubernetes](#kubernetes-k8s), [Load Balancer](#load-balancer)

---

### Health Check
**Einfach:** Automatische Prüfung, ob ein Service noch läuft und funktioniert.

**Technisch:** Automated test verifying service availability and responsiveness.

**Im Projekt:**
- **Liveness:** `GET /healthz` - Läuft der Service?
- **Readiness:** `GET /readyz` - Ist er bereit für Traffic?

**Verwendung:** Kubernetes startet Container neu bei failed Health Check.

**Verwandte Begriffe:** [Monitoring](#monitoring--observability), [Kubernetes](#kubernetes-k8s)

---

## 9. Netzwerk & Kommunikation

### TLS (Transport Layer Security) ⭐
**Einfach:** Verschlüsselung für Netzwerkverbindungen - macht HTTP zu HTTPS.

**Technisch:** Cryptographic protocol providing communications security over computer network.

**Funktionen:**
- **Verschlüsselung:** Daten sind unleserlich für Dritte
- **Authentifizierung:** Bestätigung der Server-Identität
- **Integrität:** Erkennung von Manipulationen

**Im Projekt:** rustls für TLS 1.3, optionales mTLS.

**Versionen:**
- TLS 1.2 (alt, aber weit verbreitet)
- TLS 1.3 (neu, schneller, sicherer)

**Verwandte Begriffe:** [Certificate](#certificate), [mTLS](#mtls-mutual-tls)

---

### mTLS (Mutual TLS)
**Einfach:** TLS, wo beide Seiten sich gegenseitig ausweisen (nicht nur der Server).

**Technisch:** TLS variant where both client and server authenticate each other using certificates.

**Vorteil:** Höhere Sicherheit - nur autorisierte Clients können verbinden.

**Im Projekt:** Optional für Enterprise-Deployments.

**Analogie:** Wie ein Türsteher, der auch Ihren Ausweis prüft (statt nur der Club zeigt seinen).

**Verwandte Begriffe:** [TLS](#tls-transport-layer-security), [Certificate](#certificate)

---

### Certificate (Zertifikat)
**Einfach:** Digitaler "Ausweis" für Server - bestätigt Identität.

**Technisch:** Digital document binding public key to identity, signed by Certificate Authority (CA).

**Inhalt:**
- Domain-Name
- Public Key
- Gültigkeit (von-bis)
- Signatur der CA

**Im Projekt:** X.509-Zertifikate für TLS (Let's Encrypt oder self-signed).

**Dateien:**
- `server.crt` - Zertifikat (öffentlich)
- `server.key` - Private Key (geheim!)

**Verwandte Begriffe:** [TLS](#tls-transport-layer-security), [CA](#ca-certificate-authority)

---

### CA (Certificate Authority)
**Einfach:** Vertrauenswürdige Organisation, die Zertifikate ausstellt (wie ein Notariat).

**Technisch:** Trusted entity issuing digital certificates.

**Beispiele:**
- Let's Encrypt (kostenlos)
- DigiCert, GlobalSign (kommerziell)

**Im Projekt:** cert-manager in K8s für automatische Let's Encrypt-Zertifikate.

**Verwandte Begriffe:** [Certificate](#certificate-zertifikat), [TLS](#tls-transport-layer-security)

---

### Port
**Einfach:** Nummer, die angibt, welcher Dienst auf einem Server angesprochen wird.

**Technisch:** 16-bit number identifying specific process/service on networked computer.

**Standard-Ports:**
- 80 - HTTP
- 443 - HTTPS
- 22 - SSH

**Im Projekt:**
- 8080 - HTTP (Development)
- 8443 - HTTPS (Production)

**Verwandte Begriffe:** [Socket](#socket), [Bind](#bind)

---

### Firewall
**Einfach:** Sicherheitssystem, das Netzwerk-Traffic filtert (wie ein Türsteher).

**Technisch:** Network security system monitoring and controlling incoming/outgoing traffic based on rules.

**Im Projekt:** UFW oder iptables für Firewall-Regeln (Port 8443 freigeben).

**Verwandte Begriffe:** [Port](#port), [Security](#2-kryptographie--sicherheit)

---

### Load Balancer ⭐
**Einfach:** Verteiler, der Anfragen auf mehrere Server verteilt.

**Technisch:** Device or software distributing network traffic across multiple servers.

**Algorithmen:**
- **Round Robin:** Reihum
- **Least Connections:** Wenigste aktive Verbindungen
- **IP Hash:** Basierend auf Client-IP

**Im Projekt:** Kubernetes Ingress fungiert als Load Balancer für 3 Replicas.

**Analogie:** Wie mehrere Kassen im Supermarkt - jeder Kunde geht zur freien Kasse.

**Verwandte Begriffe:** [Horizontal Scaling](#horizontal-scaling), [Kubernetes](#kubernetes-k8s)

---

### Proxy
**Einfach:** Zwischenstation für Netzwerk-Anfragen (kann filtern, cachen, anonymisieren).

**Technisch:** Intermediary server separating clients from resources they request.

**Arten:**
- **Forward Proxy:** Client → Proxy → Internet
- **Reverse Proxy:** Internet → Proxy → Server

**Im Projekt:** Nginx könnte als Reverse Proxy vor API-Server eingesetzt werden.

**Verwandte Begriffe:** [Load Balancer](#load-balancer), [Gateway](#gateway)

---

### DNS (Domain Name System)
**Einfach:** Das "Telefonbuch des Internets" - übersetzt Namen in IP-Adressen.

**Technisch:** Hierarchical decentralized naming system translating domain names to IP addresses.

**Beispiel:**
```
verifier.example.com → 192.168.1.100
```

**Im Projekt:** DNS-Einträge für Production-Deployment (A-Record oder CNAME).

**Verwandte Begriffe:** [Domain](#domain), [IP Address](#ip-address)

---

### Latency (Latenz)
**Einfach:** Verzögerung - Zeit zwischen Anfrage und Antwort.

**Technisch:** Time delay between cause and effect in system.

**Messung:** Millisekunden (ms)

**Im Projekt:** Performance-Metriken zeigen Latenz:
- P50: 18 ms (median)
- P95: 35 ms (95% der Anfragen)
- P99: 55 ms (99%)

**Ziel:** Niedrige Latenz = schnelle Antworten.

**Verwandte Begriffe:** [Throughput](#throughput), [Performance](#performance)

---

## 10. Authentifizierung & Autorisierung

### Authentication (Authentifizierung) ⭐
**Einfach:** Beweis der Identität - "Wer bist du?"

**Technisch:** Process of verifying identity of user, device, or system.

**Methoden:**
- **Passwort:** Etwas, das man weiß
- **2FA:** Zusätzlicher Faktor (z.B. SMS-Code)
- **Zertifikat:** Etwas, das man hat
- **Biometrie:** Etwas, das man ist

**Im Projekt:** OAuth2 JWT-Tokens für API-Authentifizierung.

**Verwandte Begriffe:** [Authorization](#authorization-autorisierung), [OAuth2](#oauth2)

---

### Authorization (Autorisierung)
**Einfach:** Berechtigung - "Was darfst du?"

**Technisch:** Process of verifying what user/system is allowed to do.

**Beispiel:**
- **Authentication:** Du bist "Max Mustermann" ✓
- **Authorization:** Du darfst Datei X lesen, aber nicht ändern ✓

**Im Projekt:** Scope-basierte Autorisierung (`verify:read` Scope erforderlich).

**Verwandte Begriffe:** [Authentication](#authentication-authentifizierung), [Scope](#scope)

---

### OAuth2 ⭐
**Einfach:** Standard-Verfahren für Zugriffskontrolle (wie "Mit Google anmelden").

**Technisch:** Industry-standard protocol for authorization enabling applications to obtain limited access to user resources.

**Flows:**
- **Client Credentials:** Maschine-zu-Maschine (im Projekt verwendet)
- **Authorization Code:** Benutzer-Login
- **Refresh Token:** Token-Erneuerung

**Im Projekt:** Client Credentials Flow für API-Zugriff.

**Verwandte Begriffe:** [JWT](#jwt-json-web-token), [Token](#token)

---

### JWT (JSON Web Token) ⭐
**Einfach:** Ein "digitaler Ausweis" mit Ablaufdatum, als JSON kodiert.

**Technisch:** Compact, URL-safe token format encoding claims between parties as JSON object, digitally signed.

**Struktur:** `Header.Payload.Signature`

**Header:**
```json
{
  "alg": "RS256",
  "typ": "JWT"
}
```

**Payload (Claims):**
```json
{
  "sub": "client-id",
  "iss": "auth.example.com",
  "exp": 1700000000,
  "scope": "verify:read"
}
```

**Signatur:** RS256 (RSA mit SHA-256)

**Im Projekt:** Verwendet für OAuth2-Authentifizierung.

**Verwandte Begriffe:** [OAuth2](#oauth2), [Claims](#claims)

---

### Claims
**Einfach:** Aussagen/Informationen in einem JWT-Token (wer, wann, was).

**Technisch:** Key-value pairs in JWT payload asserting information about entity.

**Standard-Claims:**
- `sub` (Subject): Wer?
- `iss` (Issuer): Von wem?
- `aud` (Audience): Für wen?
- `exp` (Expiration): Bis wann?
- `iat` (Issued At): Wann ausgestellt?

**Im Projekt:** Custom Claim `scope` für Berechtigungen.

**Verwandte Begriffe:** [JWT](#jwt-json-web-token), [OAuth2](#oauth2)

---

### Scope
**Einfach:** Berechtigungen, die ein Token hat (z.B. "darf lesen", "darf schreiben").

**Technisch:** Set of permissions granted to token.

**Format:** Space-separated String
```
"verify:read verify:write admin:*"
```

**Im Projekt:** `verify:read` Scope erforderlich für API-Zugriff.

**Verwandte Begriffe:** [OAuth2](#oauth2), [Authorization](#authorization-autorisierung)

---

### Bearer Token
**Einfach:** Token, das man "vorzeigt", um sich zu authentifizieren (wie ein Ticket).

**Technisch:** Security token sent in HTTP Authorization header with scheme "Bearer".

**Format:**
```
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Im Projekt:** Standard-Methode für API-Authentifizierung.

**Sicherheit:** ⚠️ Token wie Passwort behandeln - niemals in URLs!

**Verwandte Begriffe:** [JWT](#jwt-json-web-token), [OAuth2](#oauth2)

---

### RBAC (Role-Based Access Control)
**Einfach:** Berechtigungssystem basierend auf Rollen (z.B. "Admin", "User").

**Technisch:** Access control mechanism assigning permissions to roles rather than individuals.

**Beispiel:**
- **Admin-Rolle:** Alles erlaubt
- **Auditor-Rolle:** Nur Lesen
- **User-Rolle:** Eigene Daten

**Im Projekt:** Nicht explizit implementiert, aber über Scopes realisierbar.

**Verwandte Begriffe:** [Authorization](#authorization-autorisierung), [Scope](#scope)

---

### Session
**Einfach:** Zusammenhängende "Sitzung" eines Nutzers (vom Login bis Logout).

**Technisch:** Temporary interactive information exchange between user and system.

**Im Projekt:** NICHT verwendet - API ist stateless (JWT-Tokens statt Sessions).

**Vorteil Stateless:** Bessere Skalierbarkeit.

**Verwandte Begriffe:** [Stateless](#stateless), [Cookie](#cookie)

---

## 11. Testing & Quality Assurance

### Unit Test
**Einfach:** Test für eine einzelne Funktion/Komponente isoliert.

**Technisch:** Test verifying behavior of smallest testable unit in isolation.

**Beispiel:**
```rust
#[test]
fn test_blake3_hash() {
    let hash = blake3::hash(b"test");
    assert_eq!(hash.to_hex().len(), 64);
}
```

**Im Projekt:** 146 Tests insgesamt, viele Unit Tests für Crypto, Registry, Keys.

**Verwandte Begriffe:** [Integration Test](#integration-test), [Test Coverage](#test-coverage)

---

### Integration Test
**Einfach:** Test, der mehrere Komponenten zusammen testet.

**Technisch:** Test verifying correct interaction between multiple components/modules.

**Im Projekt:** `tests/` Verzeichnis:
- `test_bundle_v2.rs` - Bundle-Erstellung und Verifikation
- `test_integration_http.rs` - REST API Tests
- `test_registry_sqlite.rs` - Registry-Backend Tests

**Verwandte Begriffe:** [Unit Test](#unit-test), [E2E Test](#e2e-test-end-to-end)

---

### E2E Test (End-to-End)
**Einfach:** Test des gesamten Systems von Anfang bis Ende (wie ein echter Nutzer).

**Technisch:** Test validating complete workflow from user perspective.

**Beispiel:** CSV-Import → Proof-Erstellung → Verifikation → Registry-Eintrag

**Im Projekt:** Teils in Integration Tests abgedeckt.

**Verwandte Begriffe:** [Integration Test](#integration-test), [Acceptance Test](#acceptance-test)

---

### Test Coverage
**Einfach:** Prozentsatz des Codes, der von Tests abgedeckt ist.

**Technisch:** Metric measuring percentage of codebase executed by tests.

**Im Projekt:** 100% Test-Success-Rate (146/146), aber Coverage-% nicht gemessen.

**Ziel:** Hohe Coverage (> 80%), aber Qualität > Quantität.

**Tools:** `cargo tarpaulin` für Coverage-Messung.

**Verwandte Begriffe:** [Unit Test](#unit-test), [Code Quality](#code-quality)

---

### Mock
**Einfach:** Attrappe/Dummy-Objekt für Tests (simuliert echtes Verhalten).

**Technisch:** Test double replacing real dependency with simulated version.

**Im Projekt:**
- Mock Proof Backend (Phase 1)
- Mock OAuth2 Tokens (Development)

**Vorteil:** Tests ohne externe Abhängigkeiten (Datenbank, API, etc.).

**Verwandte Begriffe:** [Stub](#stub), [Test Double](#test-double)

---

### Assertion
**Einfach:** Prüfung in einem Test - "Das Ergebnis muss X sein".

**Technisch:** Statement checking expected condition in test.

**Rust-Makros:**
```rust
assert_eq!(a, b);      // a muss gleich b sein
assert!(condition);    // condition muss true sein
assert_ne!(a, b);      // a muss ungleich b sein
```

**Verwandte Begriffe:** [Unit Test](#unit-test), [Test](#testing--quality-assurance)

---

### Benchmark
**Einfach:** Geschwindigkeitsmessung von Code.

**Technisch:** Standardized test measuring performance characteristics of code.

**Im Projekt:** `benches/` Verzeichnis:
- `registry_bench.rs` - JSON vs SQLite Performance
- `compile_bench.rs` - Policy-Compiler Performance

**Tool:** Criterion crate

**Verwandte Begriffe:** [Performance](#performance), [Profiling](#profiling)

---

### Regression Test
**Einfach:** Test, der sicherstellt, dass alte Bugs nicht wiederkommen.

**Technisch:** Test verifying previously working functionality still works after changes.

**Im Projekt:** Alle Tests fungieren als Regression Tests.

**Verwandte Begriffe:** [Unit Test](#unit-test), [CI/CD](#cicd)

---

### CI/CD (Continuous Integration/Deployment) ⭐
**Einfach:** Automatisches Testen und Ausrollen von Code-Änderungen.

**Technisch:** Software development practice automatically building, testing, and deploying code changes.

**Im Projekt:** `.github/workflows/`:
- `ci.yml` - Build + Test + Audit bei jedem Commit
- `security.yml` - Security Scanning
- `release.yml` - Release Management

**Vorteile:**
- Frühe Fehler-Erkennung
- Schnellere Releases
- Konsistente Builds

**Verwandte Begriffe:** [GitHub Actions](#github-actions), [Pipeline](#pipeline)

---

## 12. Monitoring & Observability

### Prometheus ⭐
**Einfach:** System zur Sammlung und Speicherung von Metriken (Zahlen über System-Zustand).

**Technisch:** Open-source monitoring system with time-series database for metrics.

**Im Projekt:** `/metrics` Endpoint liefert Prometheus-Format.

**Metriken:**
- Counter (nur hoch)
- Gauge (hoch und runter)
- Histogram (Verteilung)

**Verwandte Begriffe:** [Metrics](#metrics), [Grafana](#grafana)

---

### Grafana
**Einfach:** Tool zur Visualisierung von Metriken als Diagramme/Dashboards.

**Technisch:** Open-source analytics and monitoring platform.

**Im Projekt:** Optional in docker-compose.yml, visualisiert Prometheus-Metriken.

**Verwandte Begriffe:** [Prometheus](#prometheus), [Dashboard](#dashboard)

---

### Metrics ⭐
**Einfach:** Zahlen/Statistiken über ein System (z.B. Anzahl Anfragen, Antwortzeit).

**Technisch:** Quantitative measurements of system behavior over time.

**Im Projekt:**
- `cap_verifier_requests_total` - Counter
- `cap_verifier_request_duration_seconds` - Histogram
- `cap_cache_hit_ratio` - Gauge

**Verwandte Begriffe:** [Prometheus](#prometheus), [Observability](#observability)

---

### Logging
**Einfach:** Aufzeichnung von Ereignissen und Fehlermeldungen in Text-Dateien.

**Technisch:** Recording of events, warnings, and errors in text format for debugging and auditing.

**Levels:**
- **ERROR:** Fehler
- **WARN:** Warnungen
- **INFO:** Informationen
- **DEBUG:** Debug-Details
- **TRACE:** Sehr detailliert

**Im Projekt:** `tracing` crate für strukturiertes Logging.

**Environment Variable:** `RUST_LOG=info`

**Verwandte Begriffe:** [Tracing](#tracing-rust), [Observability](#observability)

---

### Tracing (Rust)
**Einfach:** Modernes Logging-System in Rust mit strukturierten Daten.

**Technisch:** Application-level tracing library providing structured, async-aware logging.

**Im Projekt:** `tracing-subscriber` für Log-Formatierung.

**Vorteil:** Strukturierte Logs (JSON), Spans für Request-Tracking.

**Verwandte Begriffe:** [Logging](#logging), [Observability](#observability)

---

### Alerting
**Einfach:** Automatische Benachrichtigung bei Problemen (z.B. E-Mail, SMS).

**Technisch:** Automated notification system triggering when metrics exceed thresholds.

**Beispiel:** Alert wenn Fehlerrate > 5% oder Latenz > 1000ms.

**Im Projekt:** Nicht implementiert, aber Grafana kann Alerts definieren.

**Verwandte Begriffe:** [Prometheus](#prometheus), [Monitoring](#monitoring--observability)

---

### Observability
**Einfach:** Fähigkeit, den Zustand eines Systems von außen zu verstehen.

**Technisch:** Ability to understand internal state of system based on external outputs.

**3 Säulen:**
1. **Logs:** Was ist passiert?
2. **Metrics:** Wie viel/wie schnell?
3. **Traces:** Welcher Weg?

**Im Projekt:** Logs + Metrics implementiert, Traces geplant.

**Verwandte Begriffe:** [Monitoring](#monitoring--observability), [Telemetry](#telemetry)

---

## 13. Rust-spezifische Begriffe

### Cargo ⭐
**Einfach:** Rust's Paketmanager und Build-Tool (wie npm für Node.js).

**Technisch:** Rust's package manager and build system.

**Befehle:**
```bash
cargo build        # Kompilieren
cargo test         # Tests ausführen
cargo run          # Ausführen
cargo clippy       # Linter
cargo audit        # Sicherheits-Check
```

**Im Projekt:** `Cargo.toml` definiert Dependencies.

**Verwandte Begriffe:** [crate](#crate-rust), [Rust](#rust)

---

### crate (Rust) ⭐
**Einfach:** Ein Rust-Paket/Bibliothek (wie npm-Package).

**Technisch:** Compilation unit in Rust - library or binary.

**Im Projekt verwendet:**
- `blake3` - Hashing
- `serde` - Serialization
- `axum` - Web Framework
- `tokio` - Async Runtime

**Registry:** crates.io (wie npmjs.com)

**Verwandte Begriffe:** [Cargo](#cargo), [Dependency](#dependency)

---

### serde (Rust)
**Einfach:** Rust-Library für Serialisierung/Deserialisierung (JSON, YAML, etc.).

**Technisch:** Serialization framework providing generic interface for data structure serialization.

**Features:**
- Derive-Makros (`#[derive(Serialize, Deserialize)]`)
- Format-agnostisch (JSON, YAML, TOML, etc.)

**Im Projekt:** Überall für JSON/YAML-Handling verwendet.

**Verwandte Begriffe:** [Serialization](#serialization), [JSON](#json-javascript-object-notation)

---

### Module (Rust)
**Einfach:** Organisationseinheit für Code (wie Ordner/Dateien).

**Technisch:** Rust's system for organizing code into logical units with visibility control.

**Im Projekt:** 65+ Module in verschiedenen Kategorien (api/, crypto/, verifier/, etc.).

**Syntax:**
```rust
mod crypto;         // Deklaration
use crypto::hash;   // Import
```

**Verwandte Begriffe:** [Namespace](#namespace), [Package](#package)

---

### Trait (siehe oben)
**Einfach:** Schnittstellen-Definition in Rust.

**Technisch:** Shared behavior definition through trait bounds.

**Verwandte Begriffe:** [Interface](#interface), [Generics](#generics-rust)

---

### Lifetime (Rust)
**Einfach:** Angabe, wie lange eine Referenz gültig ist (verhindert Dangling Pointers).

**Technisch:** Scope for which reference is valid, ensuring memory safety.

**Syntax:** `'a` (Lifetime-Parameter)
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { ... }
```

**Vorteil:** Keine Null-Pointer-Fehler, keine Use-After-Free.

**Verwandte Begriffe:** [Borrow Checker](#borrow-checker), [Memory Safety](#memory-safety)

---

### Borrow Checker
**Einfach:** Rust's System, das Speicher-Fehler zur Compile-Zeit verhindert.

**Technisch:** Compile-time system enforcing ownership and borrowing rules to prevent memory errors.

**Regeln:**
- Jeder Wert hat genau einen Owner
- Es kann viele immutable Referenzen ODER eine mutable Referenz geben
- Referenzen müssen immer gültig sein

**Vorteil:** Speichersicherheit ohne Garbage Collector.

**Verwandte Begriffe:** [Lifetime](#lifetime-rust), [Ownership](#ownership-rust)

---

### Ownership (Rust)
**Einfach:** Jeder Wert gehört genau einer Variable - wenn Variable endet, wird Wert freigegeben.

**Technisch:** Memory management system where each value has single owner, automatically dropped when owner goes out of scope.

**Regeln:**
1. Jeder Wert hat einen Owner
2. Es kann nur einen Owner geben
3. Wenn Owner out of scope geht, wird Wert dropped

**Vorteil:** Keine Memory Leaks, keine Dangling Pointers.

**Verwandte Begriffe:** [Borrow Checker](#borrow-checker), [Memory Safety](#memory-safety)

---

### async/await (Rust)
**Einfach:** System für asynchrone Programmierung (mehrere Aufgaben gleichzeitig ohne Threads).

**Technisch:** Language features enabling non-blocking asynchronous programming.

**Im Projekt:** Tokio Runtime + async Axum Handlers.

**Syntax:**
```rust
async fn fetch_data() -> Result<Data> { ... }
let data = fetch_data().await?;
```

**Vorteil:** Effizient bei I/O-bound Operationen (Netzwerk, Disk).

**Verwandte Begriffe:** [Tokio](#tokio-rust), [Concurrency](#concurrency)

---

### Tokio (Rust) ⭐
**Einfach:** Async Runtime für Rust - verwaltet asynchrone Aufgaben.

**Technisch:** Asynchronous runtime providing event loop, task scheduler, and async I/O.

**Im Projekt:** Basis für Axum Web-Server.

**Features:**
- Multi-threaded Task Scheduler
- Async I/O (Netzwerk, Dateien)
- Timers, Channels

**Verwandte Begriffe:** [async/await](#asyncawait-rust), [Axum](#axum)

---

### Axum ⭐
**Einfach:** Modernes Web-Framework für Rust.

**Technisch:** Ergonomic, async web framework built on Tower and Hyper.

**Im Projekt:** Basis für REST API v0.11.0.

**Features:**
- Type-safe Routing
- Middleware-Support (Tower)
- Extractor-Pattern

**Verwandte Begriffe:** [Tokio](#tokio-rust), [Tower](#tower)

---

### Tower
**Einfach:** Framework für Middleware in Rust.

**Technisch:** Library for building robust clients and servers using composable middleware.

**Im Projekt:** Basis für Axum-Middleware (Auth, Logging, Metrics).

**Konzept:** Layers - jede Layer kann Request/Response modifizieren.

**Verwandte Begriffe:** [Axum](#axum), [Middleware](#middleware)

---

## 14. WASM & WebAssembly

### WASM (WebAssembly) ⭐
**Einfach:** Binär-Format, das im Browser (oder Server) schnell ausgeführt werden kann.

**Technisch:** Binary instruction format designed as portable compilation target for high-level languages.

**Vorteile:**
- **Schnell:** Near-native Performance
- **Sicher:** Sandboxed Execution
- **Portabel:** Läuft überall (Browser, Server, Edge)

**Im Projekt:** WASM-Bundles für Verifier (Phase 3), WASM-basierter Web-Verifier (Phase 4).

**Verwandte Begriffe:** [WASI](#wasi), [Sandboxing](#sandboxing)

---

### WASI (WebAssembly System Interface)
**Einfach:** Standard-Schnittstelle für WASM-Programme, um mit dem System zu interagieren (Dateien, Netzwerk).

**Technisch:** System interface providing capabilities beyond browser environment (filesystem, network, etc.).

**Im Projekt:** Wasmtime Runtime unterstützt WASI.

**Verwandte Begriffe:** [WASM](#wasm-webassembly), [Wasmtime](#wasmtime)

---

### Wasmtime
**Einfach:** Runtime zum Ausführen von WASM-Programmen außerhalb des Browsers.

**Technisch:** Standalone JIT compiler and runtime for WebAssembly.

**Im Projekt:** Verwendet in `wasm/loader.rs` zum Laden und Ausführen von WASM-Bundles.

**Features:**
- Resource Limits (Memory, CPU)
- WASI Support

**Verwandte Begriffe:** [WASM](#wasm-webassembly), [Runtime](#runtime)

---

### Bundle (WASM)
**Einfach:** Paket mit WASM-Code + Daten (wie ein ZIP-Archiv).

**Technisch:** Package containing WASM module and associated resources.

**Im Projekt:** CAPZ v2 Container Format für Proof-Bundles.

**Verwandte Begriffe:** [CAPZ](#capz), [WASM](#wasm-webassembly)

---

### Sandboxing
**Einfach:** Isolation einer Anwendung, damit sie keinen Schaden anrichten kann.

**Technisch:** Security mechanism separating running programs to prevent malicious or malfunctioning code from affecting system.

**Im Projekt:** WASM-Module laufen sandboxed mit Memory/Time Limits.

**Verwandte Begriffe:** [WASM](#wasm-webassembly), [Security](#2-kryptographie--sicherheit)

---

## 15. Proof-Systeme

### Proof ⭐
**Einfach:** Mathematischer Nachweis, dass eine Aussage wahr ist.

**Technisch:** Cryptographic or mathematical evidence demonstrating validity of statement.

**Im Projekt:** Compliance-Proof beweist, dass Policy-Constraints erfüllt sind.

**Datenstruktur:**
```rust
struct Proof {
    version: String,          // "proof.v0"
    type: String,            // "mock", "zkvm", "halo2"
    statement: String,       // Policy-ID
    manifest_hash: String,
    proof_data: ProofData,
    status: String,          // "ok", "fail"
}
```

**Verwandte Begriffe:** [Zero-Knowledge Proof](#zero-knowledge-proof-zkp), [Verification](#verification)

---

### Statement
**Einfach:** Die Aussage, die bewiesen werden soll (z.B. "Policy XYZ ist erfüllt").

**Technisch:** Proposition to be proven true or false.

**Im Projekt:** Policy-Hash dient als Statement-Identifier.

**Verwandte Begriffe:** [Proof](#proof), [Policy](#policy)

---

### Verifier ⭐
**Einfach:** Komponente, die Proofs überprüft.

**Technisch:** Algorithm or system verifying correctness of proofs.

**Im Projekt:**
- `verifier/core.rs` - Portable Verifier (I/O-frei)
- `verifier/mod.rs` - Package Verifier (I/O-basiert)

**Verwandte Begriffe:** [Proof](#proof), [Verification](#verification)

---

### Prover
**Einfach:** Komponente, die Proofs erstellt.

**Technisch:** Algorithm or system generating proofs.

**Im Projekt:** `proof_engine.rs` fungiert als Prover.

**Verwandte Begriffe:** [Proof](#proof), [Proof System](#proof-system)

---

### Proof System
**Einfach:** Ein System von Algorithmen zum Erstellen und Prüfen von Proofs.

**Technisch:** Set of algorithms for generating and verifying proofs.

**Im Projekt:** Trait-basierte Abstraktion für austauschbare Backends:
- Mock (Phase 1)
- Halo2 (Phase 3, geplant)
- Spartan (Phase 4, geplant)

**Verwandte Begriffe:** [Prover](#prover), [Verifier](#verifier)

---

### Mock Proof
**Einfach:** Simpler "Fake"-Proof für Tests (kein echter kryptographischer Beweis).

**Technisch:** Simplified proof for testing without cryptographic guarantees.

**Im Projekt:** Standard-Backend in Phase 1 (bis echte ZK-Backends verfügbar).

**Vorteil:** Schnell, einfach zu debuggen.

**Verwandte Begriffe:** [Proof](#proof), [Zero-Knowledge Proof](#zero-knowledge-proof-zkp)

---

### Halo2
**Einfach:** Modernes Zero-Knowledge-Proof-System (sehr effizient).

**Technisch:** Zero-knowledge proof system using PLONK arithmetization and polynomial commitments.

**Im Projekt:** Geplant für Phase 3 als Production-Ready ZK-Backend.

**Verwandte Begriffe:** [Zero-Knowledge Proof](#zero-knowledge-proof-zkp), [PLONK](#plonk)

---

### SNARK (Succinct Non-interactive ARgument of Knowledge)
**Einfach:** Kompakte Zero-Knowledge-Proofs, die sehr schnell verifiziert werden können.

**Technisch:** Proof system producing short proofs verifiable in constant time.

**Eigenschaften:**
- **Succinct:** Klein (wenige KB)
- **Non-interactive:** Kein Hin-und-Her zwischen Prover/Verifier
- **Sound:** Unmöglich zu fälschen

**Beispiele:** Groth16, PLONK, Halo2.

**Verwandte Begriffe:** [Zero-Knowledge Proof](#zero-knowledge-proof-zkp), [Halo2](#halo2)

---

## 16. Allgemeine IT-Begriffe

### Binary
**Einfach:** Ausführbare Datei (kompiliertes Programm).

**Technisch:** Executable file containing machine code.

**Im Projekt:**
- `cap` - CLI Binary
- `cap-verifier-api` - REST API Binary

**Verwandte Begriffe:** [Compilation](#compilation), [Executable](#executable)

---

### Compilation
**Einfach:** Übersetzung von Quellcode in ausführbaren Maschinen-Code.

**Technisch:** Translation of source code to machine code or intermediate representation.

**Im Projekt:** `cargo build` kompiliert Rust → Binary.

**Verwandte Begriffe:** [Binary](#binary), [Rust](#rust)

---

### Environment Variable
**Einfach:** Einstellung, die außerhalb des Programms definiert wird (z.B. für Konfiguration).

**Technisch:** Dynamic named value affecting running processes.

**Im Projekt:**
- `RUST_LOG=info` - Log-Level
- `CLAUDE_CODE_MAX_OUTPUT_TOKENS` - Token-Limit

**Syntax (Bash):**
```bash
export VARIABLE=wert
```

**Verwandte Begriffe:** [Configuration](#configuration), [Shell](#shell)

---

### Shell
**Einfach:** Kommandozeilen-Interpreter (z.B. Bash, Zsh).

**Technisch:** Command-line interpreter providing interface to operating system.

**Beispiele:**
- **Bash:** Standard auf Linux
- **Zsh:** Standard auf macOS
- **PowerShell:** Windows

**Im Projekt:** CLI-Befehle über Shell ausgeführt.

**Verwandte Begriffe:** [CLI](#cli-command-line-interface), [Terminal](#terminal)

---

### Terminal
**Einfach:** Fenster, in dem man Shell-Befehle eingibt.

**Technisch:** Text-based interface to operating system (terminal emulator).

**Synonyme:** Console, Command Prompt, Terminal Emulator.

**Im Projekt:** CLI-Nutzung über Terminal.

**Verwandte Begriffe:** [Shell](#shell), [CLI](#cli-command-line-interface)

---

### Path (Dateipfad)
**Einfach:** Adresse einer Datei im Dateisystem.

**Technisch:** String specifying location of file or directory in filesystem hierarchy.

**Arten:**
- **Absolut:** `/Users/max/file.txt` (vollständiger Pfad)
- **Relativ:** `./file.txt` (relativ zum aktuellen Verzeichnis)

**Im Projekt:** Alle Dateipfade sollten absolut sein für Reproduzierbarkeit.

**Verwandte Begriffe:** [Filesystem](#filesystem), [Directory](#directory)

---

### Working Directory (Arbeitsverzeichnis)
**Einfach:** Das Verzeichnis, in dem man sich aktuell befindet.

**Technisch:** Current directory from which commands are executed.

**Befehle:**
```bash
pwd           # Zeige aktuelles Verzeichnis
cd /path      # Wechsle Verzeichnis
```

**Im Projekt:** Wichtig für relative Pfade in CLI-Befehlen.

**Verwandte Begriffe:** [Path](#path-dateipfad), [Directory](#directory)

---

### Timeout
**Einfach:** Maximale Wartezeit, bevor eine Operation abgebrochen wird.

**Technisch:** Time limit after which operation is automatically terminated.

**Im Projekt:**
- Bash-Befehle: 120 Sekunden Default
- REST API: 30 Sekunden (konfigurierbar)

**Verwandte Begriffe:** [Latency](#latency-latenz), [Performance](#performance)

---

### Whitespace
**Einfach:** Unsichtbare Zeichen wie Leerzeichen, Tabs, Zeilenumbrüche.

**Technisch:** Characters that represent horizontal or vertical space (space, tab, newline, etc.).

**Im Projekt:** YAML-Dateien sind whitespace-sensitiv (Einrückung wichtig!).

**Verwandte Begriffe:** [YAML](#yaml-yaml-aint-markup-language), [Formatting](#formatting)

---

### Delimiter
**Einfach:** Trennzeichen zwischen Datenfeldern (z.B. Komma in CSV).

**Technisch:** Character separating fields in structured data.

**Beispiele:**
- **CSV:** Komma (`,`)
- **TSV:** Tab (`\t`)
- **Path:** Slash (`/` oder `\`)

**Im Projekt:** CSV-Dateien verwenden Komma als Delimiter.

**Verwandte Begriffe:** [CSV](#csv-comma-separated-values), [Parsing](#parsing)

---

### Parsing
**Einfach:** Analyse und Umwandlung von Text in strukturierte Daten.

**Technisch:** Process of analyzing string according to rules of formal grammar.

**Beispiel:** JSON-String → Rust-Struct

**Im Projekt:** CSV-Parsing, JSON-Parsing, YAML-Parsing.

**Verwandte Begriffe:** [Serialization](#serialization), [Grammar](#grammar)

---

### Validation
**Einfach:** Prüfung, ob Daten korrekt und vollständig sind.

**Technisch:** Process of checking data against defined rules or schema.

**Im Projekt:**
- Policy-Validierung (Schema-Check)
- Manifest-Validierung (JSON Schema)
- Input-Validierung (CSV-Felder)

**Verwandte Begriffe:** [Schema](#schema), [Constraint](#constraint)

---

### Idempotence (siehe oben)
**Einfach:** Operation kann mehrfach ausgeführt werden ohne Seiteneffekte.

**Verwandte Begriffe:** [REST](#rest-representational-state-transfer), [HTTP Methods](#http-methods)

---

### Deprecation
**Einfach:** Markierung, dass etwas veraltet ist und nicht mehr genutzt werden sollte.

**Technisch:** Status indicating feature/API should no longer be used and may be removed in future.

**Im Projekt:** Alte Versionen werden als deprecated markiert vor Entfernung.

**Verwandte Begriffe:** [API Versioning](#api-versioning), [Breaking Change](#breaking-change)

---

### Bug
**Einfach:** Fehler in Software.

**Technisch:** Error, flaw, or fault causing incorrect or unexpected results.

**Im Projekt:** 0 bekannte Bugs in v0.11.0 (146/146 Tests bestanden).

**Verwandte Begriffe:** [Testing](#testing--quality-assurance), [Debugging](#debugging)

---

### Debugging
**Einfach:** Suchen und Beheben von Fehlern.

**Technisch:** Process of finding and fixing bugs in software.

**Tools:**
- Logs analysieren
- Debugger (lldb, gdb)
- Tests schreiben

**Im Projekt:** `RUST_LOG=debug` für detaillierte Logs.

**Verwandte Begriffe:** [Bug](#bug), [Logging](#logging)

---

## Anhang: Abkürzungen & Akronyme

| Abkürzung | Bedeutung | Kategorie |
|-----------|-----------|-----------|
| **API** | Application Programming Interface | Allgemein |
| **CAP** | Compliance Attestation Proof | Projekt |
| **CLI** | Command Line Interface | Allgemein |
| **CSV** | Comma-Separated Values | Datenformat |
| **DNS** | Domain Name System | Netzwerk |
| **GC** | Garbage Collection | Memory |
| **GUI** | Graphical User Interface | Allgemein |
| **HSM** | Hardware Security Module | Security |
| **HTTP** | Hypertext Transfer Protocol | Netzwerk |
| **HTTPS** | HTTP Secure | Netzwerk |
| **JSON** | JavaScript Object Notation | Datenformat |
| **JWT** | JSON Web Token | Security |
| **K8s** | Kubernetes | Deployment |
| **KID** | Key ID | Security |
| **LkSG** | Lieferkettensorgfaltspflichtengesetz | Compliance |
| **mTLS** | Mutual TLS | Security |
| **OAuth** | Open Authorization | Security |
| **PII** | Personally Identifiable Information | Privacy |
| **REST** | Representational State Transfer | API |
| **SNARK** | Succinct Non-interactive ARgument of Knowledge | ZK |
| **SQL** | Structured Query Language | Database |
| **TLS** | Transport Layer Security | Security |
| **TSA** | Time Stamp Authority | Security |
| **UBO** | Ultimate Beneficial Owner | Compliance |
| **UUID** | Universally Unique Identifier | Allgemein |
| **WAL** | Write-Ahead Logging | Database |
| **WASM** | WebAssembly | Runtime |
| **WASI** | WebAssembly System Interface | Runtime |
| **YAML** | YAML Ain't Markup Language | Datenformat |
| **ZK** | Zero-Knowledge | Crypto |
| **ZKP** | Zero-Knowledge Proof | Crypto |

---

## Index: Alphabetische Referenz

A: [ACID](#acid), [Alerting](#alerting), [API](#api-application-programming-interface), [Assertion](#assertion), [async/await](#asyncawait-rust), [Attestation](#attestation), [Audit Trail](#audit-trail), [Authentication](#authentication-authentifizierung), [Authorization](#authorization-autorisierung), [Axum](#axum)

B: [Base64](#base64), [Bearer Token](#bearer-token), [Benchmark](#benchmark), [Binary](#binary), [BLAKE3](#blake3), [Blockchain](#blockchain), [BLOB](#blob-binary-large-object), [Borrow Checker](#borrow-checker), [Bug](#bug), [Bundle](#bundle-wasm)

C: [CA](#ca-certificate-authority), [Cargo](#cargo), [Certificate](#certificate-zertifikat), [Cipher](#cipher), [Claims](#claims), [CLI](#cli-command-line-interface), [Collision](#collision-hash-kollision), [Commitment](#commitment), [Compilation](#compilation), [Compliance](#compliance), [Consensus](#consensus), [Container](#container), [Content-Addressable Storage](#content-addressable-storage), [Content-Type](#content-type), [crate](#crate-rust), [CSPRNG](#csprng-cryptographically-secure-pseudo-random-number-generator), [CSV](#csv-comma-separated-values)

D: [Debugging](#debugging), [Deduplication](#deduplication), [Defense in Depth](#defense-in-depth-mehrschichtige-verteidigung), [Delimiter](#delimiter), [Dependency Injection](#dependency-injection), [Deprecation](#deprecation), [Digest](#digest), [Digital Signature](#signature-digitale-signatur), [Distributed Ledger](#distributed-ledger), [DNS](#dns-domain-name-system), [Docker](#docker), [Dockerfile](#dockerfile), [Due Diligence](#due-diligence-sorgfaltspflicht)

E: [E2E Test](#e2e-test-end-to-end), [Ed25519](#ed25519), [Encoding](#encoding), [Encryption](#encryption-verschlüsselung), [Endpoint](#endpoint), [Entropy](#entropy-entropie), [Environment Variable](#environment-variable)

F: [Factory Pattern](#factory-pattern), [Firewall](#firewall)

G: [Garbage Collection](#garbage-collection-gc), [Grafana](#grafana)

H: [Halo2](#halo2), [Hash](#hash--hash-funktion), [Hash Chain](#hash-chain), [Health Check](#health-check), [Hexadecimal](#hexadecimal-hex), [Horizontal Scaling](#horizontal-scaling), [HTTP](#http-hypertext-transfer-protocol), [HTTPS](#https-http-secure)

I: [Idempotent](#idempotent), [Image](#image-docker), [Index](#index-datenbank), [Ingress](#ingress-kubernetes), [Integration Test](#integration-test)

J: [JSON](#json-javascript-object-notation), [Jurisdiction](#jurisdiction), [JWT](#jwt-json-web-token)

K: [Kubernetes](#kubernetes-k8s), [KID](#kid-key-id)

L: [Latency](#latency-latenz), [Layered Architecture](#layered-architecture-schichtenarchitektur), [Lifetime](#lifetime-rust), [LkSG](#lksg-lieferkettensorgfaltspflichtengesetz), [Load Balancer](#load-balancer), [Logging](#logging)

M: [Merkle Tree](#merkle-tree-merkle-baum), [Metrics](#metrics), [Microservices](#microservices), [Middleware](#middleware), [Mock](#mock), [Mock Proof](#mock-proof), [Module](#module-rust), [Monolith](#monolith), [mTLS](#mtls-mutual-tls)

N: [Nonce](#nonce)

O: [OAuth2](#oauth2), [Observability](#observability), [Orchestrator](#orchestrator), [Ownership](#ownership-rust)

P: [Parsing](#parsing), [Path](#path-dateipfad), [Persistence](#persistence), [PersistentVolume](#persistentvolume-pv), [Pipeline](#pipeline), [Plugin Architecture](#plugin-architecture), [Pod](#pod-kubernetes), [Policy](#policy), [PolicyV2](#policyv2), [Port](#port), [Private Key](#private-key-privater-schlüssel), [Prometheus](#prometheus), [Proof](#proof), [Proof System](#proof-system), [Prover](#prover), [Proxy](#proxy), [Public Key](#public-key-öffentlicher-schlüssel)

R: [Race Condition](#race-condition), [Rainbow Table](#rainbow-table), [RBAC](#rbac-role-based-access-control), [Refcount](#refcount-reference-count), [Regression Test](#regression-test), [Registry](#registry-docker), [REST](#rest-representational-state-transfer), [RFC3339](#rfc3339), [Rust](#rust)

S: [Salt](#salt), [Sandboxing](#sandboxing), [Schema](#schema), [Scope](#scope), [Separation of Concerns](#separation-of-concerns), [serde](#serde-rust), [Serialization](#serialization), [Service](#service-kubernetes), [Session](#session), [SHA3-256](#sha3-256), [Shell](#shell), [Side-Channel Attack](#side-channel-attack), [Signature](#signature-digitale-signatur), [Smart Contract](#smart-contract), [SNARK](#snark-succinct-non-interactive-argument-of-knowledge), [SQL](#sql-structured-query-language), [SQLite](#sqlite), [Statement](#statement), [Stateless](#stateless), [Supply Chain](#supply-chain-lieferkette)

T: [Terminal](#terminal), [Test Coverage](#test-coverage), [Tier](#tier), [Timeout](#timeout), [Timestamping](#timestamping-zeitstempel), [TLS](#tls-transport-layer-security), [Tokio](#tokio-rust), [Tower](#tower), [Tracing](#tracing-rust), [Trait](#trait-rust), [Transaction](#transaction)

U: [UBO](#ubo-ultimate-beneficial-owner), [Unit Test](#unit-test), [UUID](#uuid-universally-unique-identifier)

V: [Validation](#validation), [Verifier](#verifier)

W: [WAL](#wal-write-ahead-logging), [WASM](#wasm-webassembly), [WASI](#wasi-webassembly-system-interface), [Wasmtime](#wasmtime), [Whitespace](#whitespace), [Working Directory](#working-directory-arbeitsverzeichnis)

Y: [YAML](#yaml-yaml-aint-markup-language)

Z: [Zero-Knowledge Proof](#zero-knowledge-proof-zkp)

---

**Ende des Glossars**

*Version: 1.0*
*Letzte Aktualisierung: 17. November 2025*
*Projekt: LsKG-Agent v0.11.0*
