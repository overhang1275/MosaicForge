async (page) => {
  const consoleErrors = [];
  const failedRequests = [];
  page.on('console', message => {
    if (message.type() === 'error') consoleErrors.push(message.text());
  });
  page.on('pageerror', error => consoleErrors.push(error.message));
  page.on('requestfailed', request => failedRequests.push(new URL(request.url()).pathname));
  page.on('response', response => {
    if (response.status() >= 400) failedRequests.push(`${response.status()} ${new URL(response.url()).pathname}`);
  });
  const target = new URL(page.url());
  target.hash = '';
  target.searchParams.set('qa', Date.now().toString()); // Reload the current fixture rather than a cached copy.
  await page.goto(target.href);
  const input = page.getByRole('textbox', { name: 'Nombre de la prueba' });
  const button = page.getByRole('button', { name: 'Comprobar interacción' });
  const status = page.getByRole('status');
  if (await input.getAttribute('autocomplete') !== 'off' || await input.getAttribute('name') !== 'testName') throw new Error('Test field lacks explicit autofill policy or meaningful name');
  await page.keyboard.press('Tab');
  const skip = page.getByRole('link', { name: 'Ir al contenido' });
  if (!(await skip.evaluate(element => element === document.activeElement))) throw new Error('Skip link is not first keyboard stop');
  await page.keyboard.press('Enter');
  if (!(await page.locator('#main').evaluate(element => element === document.activeElement))) throw new Error('Skip navigation failed');
  const results = [];
  for (const viewport of [{ width: 1366, height: 900 }, { width: 390, height: 844 }]) {
    await page.setViewportSize(viewport);
    await input.fill(`QA ${viewport.width}`);
    await input.focus();
    await page.keyboard.press('Tab');
    if (!(await button.evaluate(element => element === document.activeElement))) throw new Error('Keyboard order failed');
    const outline = await button.evaluate(element => getComputedStyle(element).outlineStyle);
    if (outline === 'none') throw new Error('Keyboard focus is not visible');
    await page.keyboard.press('Enter');
    const text = await status.innerText();
    if (text !== `Interacción comprobada: QA ${viewport.width}.`) throw new Error('Form did not update status');
    const layout = await page.evaluate(() => ({ width: innerWidth, scrollWidth: document.documentElement.scrollWidth }));
    if (layout.scrollWidth > layout.width) throw new Error('Horizontal overflow');
    await page.evaluate(() => scrollTo(0, 0));
    await page.screenshot({ path: viewport.width === 390 ? 'mobile-resume.png' : 'desktop-resume.png', fullPage: true });
    results.push({ viewport, text, outline, ...layout });
  }
  await page.getByRole('link', { name: 'Ver criterios de la prueba' }).click();
  if (new URL(page.url()).hash !== '#criterios') throw new Error('Anchor navigation failed');
  const contrast = await page.evaluate(() => {
    const luminance = color => color.match(/[\d.]+/g).slice(0, 3).map(Number).map(n => {
      const s = n / 255;
      return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    }).reduce((sum, n, i) => sum + n * [0.2126, 0.7152, 0.0722][i], 0);
    const ratio = (a, b) => (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
    const body = getComputedStyle(document.body);
    const link = getComputedStyle(document.querySelector('a'));
    return {
      body: ratio(luminance(body.color), luminance(body.backgroundColor)),
      link: ratio(luminance(link.color), luminance(body.backgroundColor)),
    };
  });
  if (contrast.body < 4.5 || contrast.link < 4.5) throw new Error('Measured text contrast failed');
  if (consoleErrors.length || failedRequests.length) throw new Error('Console or network failure');
  return { results, contrast, consoleErrors, failedRequests, navigation: '#criterios' };
}
