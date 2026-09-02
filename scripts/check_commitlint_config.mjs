// Controls for the line-length exemption in `commitlint.config.mjs`.
//
// The exemption exists so Dependabot's pull requests can merge. It is worth a
// test because it is the kind of rule that is easy to write too wide: an
// exemption that lets every long line through reads exactly like one that lets
// only the unwrappable ones through, and the difference only shows up months
// later as an unreadable `git log`.
//
// Run with `node scripts/check_commitlint_config.mjs`. No dependencies: this
// exercises the predicate directly, not commitlint, so it needs no install.

import { testable } from '../commitlint.config.mjs';

const { LIMIT, unwrappable } = testable;

const repeat = (word, times) => Array(times).fill(word).join(' ');

const cases = [
  // Exempt: the line is long because of a link, and there is no useful wrap
  // point inside one. All four are real lines from Dependabot commits.
  [
    true,
    'a Dependabot compare link',
    '- [Commits](https://github.com/example/kcp-sys/compare/32a6c09fc6223f54aea83981a6aa8995931d29be...d7427c22d764deb1860a7d37acc446ed5033464c)',
  ],
  [
    true,
    'a Dependabot table row',
    '| [rustls-platform-verifier](https://github.com/rustls/rustls-platform-verifier) | `0.6.2` | `0.7.0` |',
  ],
  [
    true,
    'a Dependabot sentence carrying a link',
    'Bumps the cargo-major group with 1 update in the /libs/portable directory: [md5](https://github.com/stainless-steel/md5).',
  ],
  [
    true,
    'a single token longer than the limit',
    `sha256:${'a'.repeat(120)}`,
  ],

  // Not exempt: every one of these could have been wrapped, and the rule
  // exists for exactly these.
  [false, 'unwrapped prose', repeat('word', 24)],
  [
    false,
    'prose that merely contains a short link',
    `${repeat('word', 22)} https://example.com/x and more words here`,
  ],
  [
    false,
    'a long line with a link that does not account for its length',
    `${repeat('sentence', 14)} https://example.com/`,
  ],
];

let failed = 0;
for (const [expected, name, line] of cases) {
  const actual = unwrappable(line);
  const verdict = actual === expected ? 'ok  ' : 'FAIL';
  if (actual !== expected) failed += 1;
  console.log(
    `${verdict} ${name} (${line.length} chars, exempt=${actual}, want=${expected})`,
  );
}

// The predicate must never be reached for a line inside the limit; if it were,
// a short line could be reported. Guard the caller's contract too.
if (LIMIT !== 100) {
  console.log(`FAIL the limit moved to ${LIMIT} without this test moving`);
  failed += 1;
}

if (failed > 0) {
  console.error(`\n${failed} control(s) failed`);
  process.exit(1);
}
console.log(`\nall ${cases.length} controls passed`);
