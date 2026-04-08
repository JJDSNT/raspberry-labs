1. UEFI
Objetivo

Fazer o kernel entrar pelo UEFI, pegar o que precisa, sair dos Boot Services e assumir a máquina.

UEFI mínimo

O mínimo para dizer que o kernel está UEFI-ready:

entrypoint UEFI funcional
inicializar console simples para debug
localizar framebuffer
obter mapa de memória
carregar ou embutir blobs necessários
kernel payload
ROM do emulador
disco/imagens, se precisar
montar um BootInfo
chamar ExitBootServices()
transferir controle para o kernel
kernel continuar funcionando sem depender de serviço UEFI
Critério de pronto
o kernel sobe sempre a partir do UEFI
consegue imprimir/debugar
framebuffer funciona
memória disponível é conhecida
nenhum serviço de boot é usado depois do handoff
UEFI ideal

A versão “boa” dessa camada:

BootInfo estável e pequeno
loader de blobs organizado
handoff multicore previsível
descrição clara de:
framebuffer
memória
cores
timer base
módulos carregados
separação limpa entre:
código UEFI
código de handoff
código do kernel
fallback/debug decente para falha antes do handoff
O que evitar
espalhar chamadas UEFI dentro do kernel
deixar o kernel depender de tipos/headers UEFI
misturar loader com runtime
2. Kernel
Objetivo

Ter um kernel mínimo que já seja semente da plataforma, mas suficiente para rodar o emulador sem gambiarra estrutural.

Kernel mínimo

O mínimo para rodar o emulador como workload real:

inicialização pós-handoff
scheduler preemptivo
task/thread creation
afinidade por core
timer monotônico
sleep/yield
allocator físico ou page allocator simples
IRQ básico
console/log
framebuffer/present
input básico
IPC simples:
fila
mailbox
evento/sinalização
API mínima para subsistemas:
memória
tempo
vídeo
input
log
IPC
ability de rodar uma task pinned em core separado
Critério de pronto
o kernel sobe sozinho depois do handoff
consegue criar task do emulador
consegue prender essa task a um core
consegue entregar input, tempo e vídeo ao emulador
continua sendo dono da máquina, não o emulador
Kernel ideal

A versão “ideal” para evoluir sem jogar fora:

separação clara entre:
boot/
platform/
kernel/
drivers/
subsystems/
memory regions explícitas:
RAM
MMIO
reserved
shared
noção de device
noção de MMIO region
bus/access layer inicial
SMP básico sólido:
core bring-up
core id
IPI ou mecanismo equivalente
observabilidade:
counters
status de tasks
tracing simples
KernelAPI pequena e estável para clientes
shared memory como conceito explícito, não improviso
ownership claro:
hardware real pertence ao kernel
estado interno do emulador pertence ao emulador
O que evitar
APIs “feitas para o emulador”
hacks temporais só para ele
boot code vazando para o runtime
tudo global e sem ownership
3. Emulador
Objetivo

Rodar primeiro como workload útil e depois poder descer para camada de compatibilidade.

Emulador mínimo

O mínimo para rodar sobre o kernel:

remover dependência direta de SDL/host anterior
receber uma API do kernel
init/reset/start previsíveis
loop funcional sobre task própria
input por fila/evento
vídeo por buffer compartilhado ou present callback
uso de clock do kernel
ROM carregada por blob já entregue
disco/imagem também entregue pelo kernel
rodar em core dedicado
estado encapsulado
Critério de pronto
o emulador roda sem acessar diretamente hardware real
não chama serviços do host antigo
usa só a API do kernel
recebe input do kernel
produz frame para o kernel
boota algo útil
Emulador ideal

A forma ideal para não atrapalhar o futuro:

API explícita, tipo:
init
reset
run_slice
send_command
publish_frame
shutdown
estado dividido entre:
privado
compartilhado
privado:
CPU state
chipset state
estruturas internas
compartilhado:
command queue
frame buffers
audio buffers
status page
orientado a budget/slice, não loop opaco
observável:
FPS
carga
estado
faults
preparado para deixar de ser “o centro”
integra-se ao modelo do kernel:
IPC
shared memory
timer
eventualmente bus/MMIO
O que evitar
emulador mandando no timing do sistema
emulador falando direto com hardware real
estado interno exposto para qualquer um
loop monolítico impossível de controlar
Ordem recomendada entre as três trilhas
Primeiro

UEFI mínimo

porque sem handoff limpo, todo o resto fica sujo.

Depois

Kernel mínimo

porque ele é o substrate para tudo.

Depois

Emulador mínimo

porque aí você valida o sistema inteiro com workload real.

Só então

começar a puxar cada um para o “ideal”:

primeiro kernel
depois emulador
UEFI só o suficiente para não atrapalhar
Resumo curto
UEFI mínimo
bootar
framebuffer
memória
blobs
handoff limpo
Kernel mínimo
scheduler
core affinity
timer
memória
IRQ
vídeo/input
IPC
API para subsistemas
Emulador mínimo
sem SDL/host antigo
task própria
input do kernel
vídeo para o kernel
ROM/disco via kernel
roda em core dedicado
UEFI ideal
handoff pequeno, limpo, descartável
Kernel ideal
base arquitetural da máquina
Emulador ideal
workload modular hoje, compat layer amanhã