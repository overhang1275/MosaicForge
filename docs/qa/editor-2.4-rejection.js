async (page) => {
  await page.getByLabel('Variantes de muestra',{exact:true}).fill('1');
  await page.getByText('Transformaciones',{exact:true}).click();
  const opacity=page.getByRole('group',{name:'Opacidad',exact:true});
  await opacity.getByLabel('Mínimo',{exact:true}).fill('0');
  await opacity.getByLabel('Máximo',{exact:true}).fill('0');
  await page.getByRole('button',{name:'Generar variantes',exact:true}).click();
  await page.getByRole('heading',{name:'Variante rechazada',exact:true}).waitFor({state:'visible',timeout:40000});
  if(await page.locator('.preview-canvas img').count()!==0 || await page.getByRole('button',{name:/Descargar (SVG|PNG)/}).count()!==0)throw new Error('Rejected variant must not expose artifact');
  await page.screenshot({path:'C:/Users/init/Desktop/MosaicForge/out/editor-qa/rejected-final.png'});
  return {message:await page.locator('.preview-canvas').innerText(),downloads:0,images:0};
}
