🧠 Visão do Projeto (reformulada)
1. Ideia central

O projeto define uma plataforma arquitetural própria, implementada diretamente por um kernel, que serve como base para um sistema moderno inspirado no Amiga, com o AROS como sistema final.

Essa plataforma não é uma VM no sentido clássico.

Ela é:

uma máquina lógica implementada pelo kernel

2. Natureza da “máquina”

A máquina MC68k-64 não é:

um guest isolado
nem uma VM tradicional
nem uma camada separada

Ela é:

o modelo arquitetural que o kernel implementa diretamente

Isso inclui:

modelo de memória
mapa físico
devices/MMIO
IRQ
DMA
execução
3. Papel do kernel

O kernel não é apenas infraestrutura.

Ele é:

a implementação concreta da máquina

Responsabilidades:

controle de execução (scheduler, SMP)
tempo
memória física e regiões
IPC
devices
MMIO / bus
integração entre componentes

Ou seja:

kernel == máquina
4. Papel do emulador

O emulador não é a máquina.

Ele é:

o primeiro workload relevante do sistema

Funções:

validar o kernel
fornecer software real desde o início
exercitar:
SMP
memória
IPC
vídeo/input
timing
servir como base de compatibilidade futura

No futuro:

ele deixa de ser central e vira camada de compatibilidade

5. Papel do AROS

O AROS não é um guest de VM.

Ele é:

o sistema operacional principal da plataforma

Mas não sobre hardware cru — e sim sobre a máquina definida pelo kernel.

A longo prazo:

o AROS se torna o ator principal, integrado à base arquitetural criada

6. Relação entre os componentes
Fase inicial
hardware → kernel → emulador
Fase intermediária
hardware → kernel (máquina) → emulador + serviços
Fase alvo
hardware → kernel (máquina) → AROS
                           → compatibilidade (emulador)
7. Princípio fundamental

A arquitetura define o sistema — não o mecanismo de execução

Por isso:

VM não é necessária
virtualização é opcional (futuro)
o kernel já implementa os contratos arquiteturais
8. Influência do PiStorm

Do PiStorm vem um princípio importante:

execução heterogênea com memória compartilhada e integração direta

Isso se traduz na plataforma como:

múltiplos “atores” no sistema
memória compartilhada controlada
integração via:
IPC
MMIO
eventos
9. Objetivo de longo prazo

Criar uma plataforma que permita ao AROS evoluir além das limitações atuais, mantendo coerência arquitetural desde a base.

Isso inclui:

SMP desde o início
MMU desde o design
modelo de memória consistente
devices modernos
integração com hardware atual
10. O que o projeto NÃO é

Para evitar ambiguidade:

❌ não é uma VM clássica
❌ não é um hypervisor-first
❌ não é “rodar AROS em cima de um kernel genérico”
❌ não é “um emulador com sistema ao redor”
11. Definição final (curta)

O projeto é a construção de uma máquina arquitetural moderna, implementada diretamente por um kernel, que usa um emulador como workload inicial e evolui para ter o AROS como sistema principal, mantendo coerência estrutural desde o início.