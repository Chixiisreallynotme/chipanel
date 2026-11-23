# Design Council VI — Unification UI/UX « un seul code partout »

**Date :** 2026-11-23
**Statut :** spec validée par délégation de décision (réponses de cadrage : « audit puis réduire le code », « recherche puis brainstorm et choisi », « décide toi même »).
**Périmètre :** `frontend/` uniquement. Aucun endpoint, aucun contrat de données, aucun backend.

---

## 1. Constat (mesuré, pas supposé)

Audit croisé source + application live (`http://localhost:25500`, 20 routes, Playwright, 1440/390) :

- **Déjà unifié** : 15/15 routes cockpit = `PageHeader` + `.page-stack`, h1 28px/700, `--bg-base #0B0D13`, zéro overflow horizontal, un seul barrel d'icônes, `check:tokens` OK (121 tokens / 0 drift), `Modal.svelte` a11y en place.
- **Dérive réelle = duplication de CSS scopé** : `.active` ×23, `.modal-header` ×14 (11 variantes), `.loading-state` ×11 (11 variantes), `.tabular-nums` ×12 (1 variante, déjà globale), `.spinner` ×10, `@keyframes spin` ×25 (3 variantes, dont une incomplète), `.empty-icon` ×15 (13 variantes), `.data-table` ×13 (9 variantes).
- **Incohérences structurelles** : 3 wrappers locaux morts (`players/worlds/permissions-page` = `position: relative`), 2 wrappers plein écran dupliqués (`console`/`files`), `/setup` = 2 `<main>` imbriqués + 0 `<h1>` + premier titre en H2 + `ModeSwitch` affiché deux fois, `lang="en"` sur une UI française, 18 `<title>` avec 4 formats et 2 titres génériques manquants, `favicon.svg` 404 alors que `favicon.png` existe, cibles tactiles header de 24–32px à 390px.

## 2. Recherche d'outils et décision

| Outil | Verdict |
| :--- | :--- |
| Tailwind / UnoCSS | Réécriture, runtime supplémentaire, conflit avec 121 tokens et « zéro framework CSS lourd » (`PRODUCT.md`). **Écarté.** |
| Stylelint `no-duplicate-selectors` | Ne détecte les doublons **que dans un seul fichier** ; plugin custom + devDep nécessaires. **Écarté.** |
| Docs Svelte 5 (`scoped-styles` / `global-styles`, Context7 `/sveltejs/svelte`) | `<style>` scopé = règles propres au composant ; décisions partagées = CSS global + custom properties. Confirme la séparation. |
| `@layer reset/tokens/components/utilities` (css.dev) | Spécificité explicite, mais changer la cascade globale à ce stade est risqué. **Non retenu.** |
| `check-tokens.mjs` existant | Zéro dépendance, déjà branché sur `npm run check`, déjà la culture du projet (`design.md` §16.4). **Retenus : l'étendre.** |

**Approche retenue** (parmi 3) : couche globale canonique + suppression des copies + garde-fou anti-régression par baseline. Ni refonte totale (trop risquée sans runner de test), ni simple documentation (aucune réduction).

## 3. Décisions prises

- **Langue** : français partout. `<html lang="fr">`, titres de page et h1 traduits.
- **Titres** : format unique `Libellé | ChiPanel`, un `<title>` par route, aucun tiret cadratin (R-02), suffixe unique (plus de `- ChiServ`). La convention est vérifiée par le garde-fou, pas par un helper : une chaîne par route reste plus simple qu'une abstraction pour 20 usages.
- **Assets** : aucun asset créé. `favicon.svg` 404 corrigé en pointant sur `static/favicon.png` (existant) et sur le manifeste (R-23 respecté).
- **A11y** : `/setup` = 1 `<main>`, 1 `<h1>` (`.sr-only`), H2 conservé, une seule bascule de mode (celle du header global). Cibles header ≥ 44px à 390px, sans overflow.
- **Variantes locales légitimes** (`.active`, hauteurs plein écran, séries uPlot, `.loading-state` contextuels) : conservées, mais enregistrées dans la baseline du garde-fou pour interdire toute **nouvelle** divergence.

## 4. Architecture cible

```
frontend/src/
├── app.css                      # suite globale + primitives canoniques + @keyframes spin unique
├── lib/styles/tokens.css        # 121 tokens + .tabular-nums + .page-stack + .page-stack--full
├── lib/utils/                   # (aucun helper de titre : convention garde-fou)
├── scripts/
│   ├── check-tokens.mjs         # + contrôle des sélecteurs canoniques, des <title>, de lang
│   └── ui-baseline.json         # empreinte des variantes locales acceptées
└── app.html                     # lang=fr, favicon.png, theme-color aligné sur --bg-base
```

Règle d'extension : toute classe partagée est définie **une fois** dans `app.css`/`tokens.css`. Un `.svelte` ne peut redéfinir un sélecteur canonique que si le fichier figure dans `ui-baseline.json` ; sinon `npm run check` échoue.

## 5. Étapes et critères de vérification

| # | Étape | Vérification |
| :-- | :--- | :--- |
| 1 | `@keyframes spin` global unique + suppression des 25 copies | `check:tokens`, `npm run check`, spinner animé constaté en live |
| 2 | Suppression des 11 copies de `.tabular-nums` | `npm run check`, `tabular-nums` toujours calculé sur les métriques |
| 3 | Primitives canoniques globale (`.sr-only`, `.btn-md`, utilitaires de couleur/état) | `check:tokens` : 0 drift |
| 4 | Wrappers : 3 morts supprimés, 2 plein écran → `.page-stack--full` | Playwright : 20 routes, zéro overflow |
| 5 | `/setup`, `lang`, favicon, ModeSwitch, cibles tactiles | Playwright : 1 `<main>`, 1 `<h1>`, `lang=fr`, tap ≥ 44px, 0 erreur console |
| 6 | 20 `<title>` au format unique | garde-fou + Playwright |
| 7 | Garde-fou étendu + baseline | `npm run check:tokens` échoue sur une copie injectée, puis passe après retrait |
| 8 | `design.md` §17 + `CLAUDE.md` (mine de savoir) | docs commitées |
| 9 | Build + click-through + Delivery Gate antislop | `npm run build` exit 0, rapport PASS/FAIL, commit + push |

## 6. Hors périmètre (assumé)

- Les `rgba(...)` d'accent littéraux (dette §16.7).
- Les couleurs de séries `MetricsChart` (JS uPlot).
- La migration de chaque modale vers les slots de `Modal.svelte`.
- `confirm()`/`alert()` natifs : déjà absents.
- Le bruit 401 pré-auth et Google Fonts bloquant dans `<head>` : correctifs transverses, reportés.
- Les variants `.modal-*` / `.loading-state` / `.data-table` non identiques : conservés et verrouillés par baseline, à traiter dans une passe dédiée.

## 7. Risques identifiés

1. **Suppression d'un `@keyframes` local** : Svelte ne renomme une animation que si le keyframe est déclaré localement ; sans déclaration locale, `animation: spin …` référence le keyframe global. À confirmer par `getComputedStyle().animationName` en live.
2. **Spécificité** : un sélecteur scopé Svelte (0-1-0) bat une règle globale. Supprimer une copie locale sans laisser de règle concurrente est impératif.
3. **Header mobile** : porter 7 contrôles à 44px peut déborder à 390px. Contrôle par Playwright, ajustement si nécessaire.
4. **Preview stale** : l'image `:25500` n'est pas reconstruite par cette passe ; la vérification live se fait sur `npm run dev` (`:5173`, proxy `/api` → `:25500`).
