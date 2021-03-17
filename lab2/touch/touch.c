

#include "stm32f407xx.h"
#include "system_stm32f4xx.h"

static int stopped = 0;
static uint16_t i = 1;

/*************************************************
* function declarations
*************************************************/
int main(void);
void delay(volatile uint32_t);


/*************************************************
* external interrupt handler
*************************************************/
void EXTI9_5_IRQHandler(void)
{
    // Check if the interrupt came from exti1
    if (((EXTI->PR & 0x80) >> 7) == 1){
        
        RCC->APB1ENR ^= (1 << 0);
        // if (stopped)
        //     NVIC_EnableIRQ(TIM2_IRQn);
        // else
        //     NVIC_DisableIRQ(TIM2_IRQn);
        // stopped = stopped ? 0 : 1;

        // Clear pending bit
        EXTI->PR |= (1 << 7);
    }
}

/*************************************************
* timer 2 interrupt handler
*************************************************/
void TIM2_IRQHandler(void)
{
    // clear interrupt status
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }
    GPIOD->ODR = (i << 12);
    if (i == 0x08) {
        i = 1;
    }
    else {
        i = (i << 1);
    }
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 168 Mhz */
    set_sysclk_to_168();
 
    /* Enable GPIOC and GPIOD clocks */
    RCC->AHB1ENR |= (RCC_AHB1ENR_GPIOCEN | RCC_AHB1ENR_GPIODEN);

    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    // set LED moder bits to 01 (output)
    GPIOD->MODER &= 0x00FFFFFF;   // Reset bits to clear old values
    GPIOD->MODER |= 0x55000000;   // Set MODER bits to 01

    /* set up PC7 */
    GPIOC->MODER &= ~(3U << 7*2); // Reset bits 15:14 to clear old values
    GPIOC->MODER |= (1 << 7*2);   // Make PC7 an output
    GPIOC->ODR |= (1 << 7);       // Charge capacitor
    GPIOC->MODER &= ~(3U << 7*2); // Reset bits 15:14 to clear old values
    GPIOC->MODER |= (0 << 7*2);   // Make PC7 an input
    
    /* tie PC7 to EXTI7 */
    SYSCFG->EXTICR[1] |= (1U << 13); // Write 0010 to map PC[7] to EXTI7 in EXTICR2

    EXTI->FTSR |= (1U << 7);   // Enable falling edge trigger on EXTI7

    EXTI->IMR |= (1U << 7);    // Mask EXTI7

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI9_5_IRQn, 1); // Priority level 1

    // // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI9_5_IRQn);

    // enable TIM2 clock (bit0)
    RCC->APB1ENR |= (1 << 0);

    // Timer clock runs at ABP1 * 2
    //   since ABP1 is set to /4 of fCLK
    //   thus 168M/4 * 2 = 84Mhz
    // set prescaler to 8399
    //   it will increment counter every prescalar cycles
    // fCK_PSC / (PSC[15:0] + 1)
    // 84 Mhz / 8399 + 1 = 10 khz timer clock speed
    TIM2->PSC = 8399;

    // Set the auto-reload value to 10000
    //   which should give 1 second timer interrupts
    TIM2->ARR = 5000;

    // Update Interrupt Enable
    TIM2->DIER |= (1 << 0);

    NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM2_IRQn);

    // Enable Timer 2 module (CEN, bit0)
    TIM2->CR1 |= (1 << 0);


    // the code should never leave its master loop, hence while(1) or for(;;)
    while(1)
    {
        // Do nothing
    }

    __asm("NOP"); // Assembly inline can be used if needed
    return 0;
}