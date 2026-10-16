// Local Windows QA fixture for Playwright CLI; run from this repository root.
async (page) => {
  const root = 'C:/Users/init/Desktop/MosaicForge';
  const errors = [], failed = [], exports = [], checks = [];
  const ensure = (value, message) => { if (!value) throw new Error(message); };
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', entry => { if (entry.type() === 'error') errors.push(entry.text()); });
  page.on('response', response => {
    if (response.status() >= 400) failed.push({path:new URL(response.url()).pathname,status:response.status()});
  });
  page.on('request', request => {
    if (new URL(request.url()).pathname === '/api/export') exports.push(JSON.parse(request.postData()).generation);
  });
  await page.setViewportSize({width:1366,height:900});
  await page.goto('http://127.0.0.1:3000/');
  await page.keyboard.press('Tab');
  ensure(await page.getByRole('link',{name:'Ir a la vista previa'}).evaluate(el=>el===document.activeElement),'skip link must be first keyboard stop');
  await page.keyboard.press('Enter');
  ensure(await page.locator('#preview-title').evaluate(el=>el===document.activeElement),'skip link must focus heading');
  checks.push('keyboard skip link');
  await page.screenshot({path:root+'/out/editor-qa/empty-final.png'});
  await page.locator('input[type="file"]').setInputFiles(root+'/test.svg');
  ensure(!await page.getByRole('button',{name:'Generar variantes',exact:true}).isEnabled(),'selection must not import automatically');
  await page.getByRole('button',{name:'Importar plantilla',exact:true}).click();
  await page.locator('.error-message').waitFor({state:'visible',timeout:30000});
  const originalError=await page.locator('.error-message').innerText();
  ensure(originalError.length>0,'original must show import error');
  await page.screenshot({path:root+'/out/editor-qa/original-error-final.png'});
  checks.push('original test.svg rejected explicitly');
  await page.locator('input[type="file"]').setInputFiles(root+'/out/test-prepared.svg');
  await page.getByRole('button',{name:'Importar plantilla',exact:true}).click();
  await page.getByText('Plantilla importada. Ajusta las reglas y genera variantes.',{exact:true}).waitFor({state:'visible',timeout:30000});
  await page.getByLabel('Variantes de muestra',{exact:true}).fill('3');
  await page.getByLabel('Ancho (px)',{exact:true}).fill('256');
  await page.getByLabel('Alto (px)',{exact:true}).fill('256');
  await page.getByRole('button',{name:'Generar variantes',exact:true}).click();
  await page.getByText('Serie terminada. Selecciona una variante para revisarla.',{exact:true}).waitFor({state:'visible',timeout:45000});
  ensure(await page.locator('.gallery-grid img').count()===3,'three real accepted previews required');
  await page.waitForFunction(()=>document.querySelector('.preview-canvas img')?.naturalWidth===256);
  ensure(await page.locator('.error-message').count()===0,'generation must finish without technical error');
  await page.screenshot({path:root+'/out/editor-qa/desktop-final.png'});
  checks.push('prepared test.svg: three real PNG previews 256x256');
  const desktop=await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth,fonts:{body:getComputedStyle(document.body).fontSize,heading:getComputedStyle(document.querySelector('h1')).fontSize},canvasBackground:getComputedStyle(document.querySelector('.preview-canvas')).backgroundColor,filter:getComputedStyle(document.querySelector('.preview-canvas img')).filter}));
  ensure(desktop.width===desktop.scroll,'desktop overflow');
  // The displayed card must retain its original configuration after editing controls.
  await page.getByLabel('Semilla',{exact:true}).fill('2027');
  for(const format of ['SVG','PNG']) {
    const pending=page.waitForEvent('download');
    await page.getByRole('button',{name:`Descargar ${format} de la variante 0`,exact:true}).first().click();
    const download=await pending;
    await download.saveAs(root+`/out/editor-qa/test-ui-0.${format.toLowerCase()}`);
  }
  ensure(exports.length===2 && exports.every(spec=>spec.seed==='2026' && spec.startIndex==='0'),'download must use saved snapshot');
  checks.push('SVG/PNG downloaded from seed 2026 after controls changed to 2027');
  await page.setViewportSize({width:390,height:844});
  await page.screenshot({path:root+'/out/editor-qa/mobile-final.png',fullPage:true});
  const mobile=await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth,preview:document.querySelector('.workspace').getBoundingClientRect().top,inspector:document.querySelector('.inspector').getBoundingClientRect().top,gallery:document.querySelector('.gallery').getBoundingClientRect().top}));
  ensure(mobile.width===mobile.scroll && mobile.preview<mobile.inspector && mobile.inspector<mobile.gallery,'mobile must fit and prioritize preview');
  checks.push('390x844: no horizontal overflow; preview-controls-gallery order');
  await page.setViewportSize({width:683,height:450});
  ensure(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'200% equivalent viewport overflow');
  checks.push('683x450 equivalent viewport fits');
  await page.setViewportSize({width:1366,height:900});
  await page.getByLabel('Semilla',{exact:true}).fill('18446744073709551615');
  await page.getByLabel('Índice inicial',{exact:true}).fill('18446744073709551615');
  await page.getByLabel('Variantes de muestra',{exact:true}).fill('2');
  await page.getByRole('button',{name:'Generar variantes',exact:true}).click();
  await page.locator('.error-message').waitFor({state:'visible'});
  ensure(/u64/.test(await page.locator('.error-message').innerText()),'overflow must be explained');
  checks.push('u64 overflow rejected before generating');
  return {checks,originalError,desktop,mobile,exports,errors,failed};
}
