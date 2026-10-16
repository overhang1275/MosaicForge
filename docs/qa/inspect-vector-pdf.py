import sys,json,hashlib
from pathlib import Path
from collections import Counter
from pypdf import PdfReader
from pypdf.generic import ContentStream
import pypdfium2 as pdfium
pdf=Path(sys.argv[1]); output=Path(sys.argv[2]); output.mkdir(parents=True,exist_ok=True)
reader=PdfReader(pdf); counts=Counter(); seen=set()
def visit(resources):
    if not resources:return
    for ref in resources.get('/XObject',{}).values():
        obj=ref.get_object(); identity=(getattr(ref,'idnum',None),getattr(ref,'generation',None))
        if identity in seen:continue
        seen.add(identity)
        subtype=str(obj.get('/Subtype'));counts[subtype]+=1
        if subtype=='/Form':
            for _,op in ContentStream(obj,reader).operations:counts['op:'+op.decode('ascii')]+=1
            visit(obj.get('/Resources'))
for page in reader.pages:
    visit(page.get('/Resources'))
    for _,op in page.get_contents().operations:counts['op:'+op.decode('ascii')]+=1
assert len(reader.pages)==(int(sys.argv[3]) if len(sys.argv)>3 else 3)
assert counts['/Image']==0
assert sum(counts['op:'+op] for op in ['m','l','c','re'])>0
boxes=[]
for page in reader.pages:
    box=[float(x) for x in page.mediabox]; boxes.append(box)
    assert abs((box[2]-box[0])*25.4/72-100)<0.001
    assert abs((box[3]-box[1])*25.4/72-100)<0.001
doc=pdfium.PdfDocument(str(pdf))
for n in range(len(doc)):
    image=doc[n].render(scale=2).to_pil(); image.save(output/f'page-{n+1}.png')
result={'pages':len(reader.pages),'bytes':pdf.stat().st_size,'sha256':hashlib.sha256(pdf.read_bytes()).hexdigest(),'boxesPt':boxes,'resourcesAndOperators':dict(counts),'renderedPages':len(doc)}
(output/'inspection.json').write_text(json.dumps(result,indent=2),encoding='utf8');print(json.dumps(result))
