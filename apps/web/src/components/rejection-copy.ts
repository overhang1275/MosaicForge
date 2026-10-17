/** Presentation only: engine reason codes remain unchanged in result metadata. */
export function rejectionCopy(reason: string): string {
  switch (reason) {
    case 'frame_exceeds_scale_limits':
      return 'El encuadre necesita más escala. Aumenta la escala máxima o reduce la rotación y vuelve a generar.';
    case 'empty_corners':
      return 'Quedaron esquinas sin cubrir. Ajusta la escala, la rotación o la traslación y vuelve a generar.';
    case 'sparse_viewport':
      return 'La composición no cubre suficiente superficie. Ajusta el encuadre o el recorte y vuelve a generar.';
    case 'empty_viewport':
      return 'Hay muy poco contenido visible. Revisa el encuadre y la opacidad de la plantilla y vuelve a generar.';
    case 'invalid_geometry':
      return 'La transformación produjo geometría no válida. Revisa los rangos de transformación y vuelve a generar.';
    default:
      return 'La variante no superó la validación del motor. Revisa los ajustes y consulta el código en Información de la variante.';
  }
}
