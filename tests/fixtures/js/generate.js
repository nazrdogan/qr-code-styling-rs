const { JSDOM } = require('jsdom');
const { QRCodeStyling } = require('qr-code-styling/lib/qr-code-styling.common.js'); // npm i qr-code-styling@1.9.2 jsdom; node generate.js .
const fs = require('fs');
const out = process.argv[2];
const types = ['square','dots','rounded','extra-rounded','classy','classy-rounded'];
const cases = [];
for (const t of types) cases.push({ name: `fromdots_${t}`, dots: { type: t, color: '#1d4ed8' } });
cases.push({ name: 'inherit_square', dots: { type: 'rounded', color: '#000000' },
  cornersSquareOptions: { type: 'extra-rounded', color: '#b91c1c' }, cornersDotOptions: { type: 'dot' } });
cases.push({ name: 'round_bg', dots: { type: 'square', color: '#000000' }, backgroundOptions: { color: '#fde68a', round: 1 } });
(async () => {
  for (const c of cases) {
    const o = { jsdom: JSDOM, type: 'svg', width: 300, height: 300, margin: 10, data: 'https://example.com/js-corners',
      qrOptions: { errorCorrectionLevel: 'Q' }, dotsOptions: { ...c.dots, roundSize: true },
      backgroundOptions: c.backgroundOptions || { color: '#ffffff' } };
    if (c.cornersSquareOptions) o.cornersSquareOptions = c.cornersSquareOptions;
    if (c.cornersDotOptions) o.cornersDotOptions = c.cornersDotOptions;
    const qr = new QRCodeStyling(o);
    fs.writeFileSync(`${out}/${c.name}.svg`, await qr.getRawData('svg'));
  }
  console.log(cases.length + ' JS svgs');
})().catch(e => { console.error(e); process.exit(1); });
