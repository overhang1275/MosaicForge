async (page) => {
  const lum = text => {
    const c = text.match(/[\d.]+/g).slice(0,3).map(Number).map(v => { v/=255; return v<=0.04045 ? v/12.92 : ((v+0.055)/1.055)**2.4; });
    return c[0]*0.2126+c[1]*0.7152+c[2]*0.0722;
  };
  const contrast = (a,b) => (Math.max(lum(a),lum(b))+0.05)/(Math.min(lum(a),lum(b))+0.05);
  await page.mouse.move(1,1);
  const styles = await page.evaluate(() => {
    const field=getComputedStyle(document.querySelector('input[name="semilla"]'));
    const label=getComputedStyle(document.querySelector('.field'));
    const primary=getComputedStyle(document.querySelector('.button.primary'));
    const secondary=getComputedStyle(document.querySelector('.button.secondary'));
    return {field:{color:field.color,bg:field.backgroundColor,border:field.borderTopColor},label:label.color,primary:{color:primary.color,gradient:primary.backgroundImage},secondary:{color:secondary.color,bg:secondary.backgroundColor,border:secondary.borderTopColor}};
  });
  const endpoints=styles.primary.gradient.match(/rgb\([^)]+\)/g);
  const ratios={fieldText:contrast(styles.field.color,styles.field.bg),fieldBorder:contrast(styles.field.border,styles.field.bg),labelOnPanel:contrast(styles.label,'rgb(14,21,38)'),primaryEndpointMinimum:Math.min(...endpoints.map(color=>contrast(styles.primary.color,color))),secondaryText:contrast(styles.secondary.color,styles.secondary.bg),secondaryBorder:contrast(styles.secondary.border,styles.secondary.bg)};
  if(ratios.fieldBorder<3 || ratios.secondaryBorder<3 || ratios.fieldText<4.5 || ratios.labelOnPanel<4.5 || ratios.primaryEndpointMinimum<4.5 || ratios.secondaryText<4.5)throw new Error(JSON.stringify(ratios));
  await page.getByLabel('Semilla',{exact:true}).fill('2026');
  await page.getByLabel('Índice inicial',{exact:true}).fill('0');
  await page.getByLabel('Variantes de muestra',{exact:true}).fill('3');
  await page.locator('.inspector-scroll').evaluate(el=>{el.scrollTop=0;});
  await page.locator('.workspace').evaluate(el=>{el.scrollTop=0;});
  await page.screenshot({path:'C:/Users/init/Desktop/MosaicForge/out/editor-qa/desktop-final.png'});
  return {ratios,styles,note:'Representative enabled controls only; gradient endpoints, not every interpolated pixel/state.'};
}
