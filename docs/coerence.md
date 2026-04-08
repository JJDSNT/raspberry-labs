vc parou em ● Agora que li o blitter completo, o diagnóstico:

  O que está errado:
  1. Shift = 0: previousA << (16-0) e >> (16-0) são UB em C (shift de 16 em uint16_t)
  2. Máscara antes do shift (errado): AFWM/ALWM aplicadas a channelA antes do barrel shift, e o valor mascarado guardado como previousA. Na hardware (e no vAmiga), o shift register guarda o dado cru (não
  mascarado); a máscara é aplicada ao resultado do shift
  3. Overflow signed: as expressões (uint16_t) << 16 passam por int e podem virar UB com valores altos

  Fix: reordenar para shift→mask, usar uint32_t explícito, salvar previousA = raw.