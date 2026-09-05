# Almendros — instrucciones para Claude Code

Cuaderno de carretes de fotografía analógica. **Todo el programa es `index.html`**: CSS,
HTML y un único script vanilla. Sin dependencias, sin compilar, sin red en ejecución.

**Cambios sueltos: apúntalos, no los hagas.** Cuando te pida un cambio suelto en mitad de
una conversación, no lo ejecutes: añádelo a una lista y responde «apuntado», nada más. Los
cambios se hacen todos juntos, y solo cuando yo te diga. Si te pido ver la lista, enséñala.

**Antes de tocar nada, lee `ARQUITECTURA.md`**: el modelo de datos, las cuatro piezas del
motor, los invariantes que no se pueden romper y el lenguaje visual. Después,
`CHANGELOG.md` para saber qué cambió en cada versión.

**Comprobación mínima antes de dar algo por bueno:** abre `index.html#pruebas` y comprueba
que las 21 comprobaciones salen en verde. Si tocas la fusión, las migraciones o el CSV,
añade una prueba nueva ahí mismo.

**Al terminar una sesión de trabajo:** si el cambio se nota al usar la aplicación, añádelo a
`CHANGELOG.md` y sube `VERSION` en `index.html` (el aviso de pestaña vieja depende de ese
número).

**Si cambia el diseño**, rehaz las capturas con `python capturas/generar.py` y comprueba que
los pies de foto de `README.md` y `LEEME.md` siguen describiendo lo que se ve.

**Documentación de cara al público:** `README.md` (inglés) y `LEEME.md` (español). Son
espejo: llevan las mismas capturas y dicen lo mismo, así que si tocas uno, toca el otro.
Cortos a propósito — lo que no cabe ahí va a `ARQUITECTURA.md` o a `escritorio/README.md`.

Aquí no entran datos reales de nadie: el repositorio solo lleva los carretes inventados de
`datosDemo()`.
