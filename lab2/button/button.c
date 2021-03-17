#include "stm32f4xx.h"
#include "system_stm32f4xx.h"

/*************************************************
* function declarations
*************************************************/
int main(void);
int stopped = 0;
static uint32_t i = 1;

/*************************************************
* external interrupt handler
*************************************************/
void EXTI0_IRQHandler(void)
{
    // Check if the interrupt came from exti0
    if (EXTI->PR & (1 << 0)){

        if ((GPIOA->IDR & 0x0001) == 1)
        {
            stopped = 1;
        }
        else 
        {
            i = 1;
            stopped = 0;
            GPIOD->ODR = (1U << 12);
        }
        // Clear pending bit
        EXTI->PR = (1 << 0);
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
    if (stopped == 0)
    {
        GPIOD->ODR = (i << 12);
        if (i == 0x08) {
            i = 1;
        }
        else {
            i = (i << 1);
        }
    }
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 168 Mhz */
    set_sysclk_to_168();

    // setup LEDs
    RCC->AHB1ENR |= (1 << 3);
    GPIOD->MODER &= 0x00FFFFFF;
    GPIOD->MODER |= 0x55000000;
    GPIOD->ODR = 0;

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


    /* set up button */
    // enable GPIOA clock (AHB1ENR: bit 0)
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOAEN; // (1 << 0);
    GPIOA->MODER &= 0xFFFFFFFC;   // Reset bits 0-1 to clear old values
    GPIOA->MODER |= 0x00000000;   // Make button an input

    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    /* tie push button at PA0 to EXTI0 */
    SYSCFG->EXTICR[0] |= 0x00000000; // Write 0000 to map PA0 to EXTI0

    // Choose either rising edge trigger (RTSR) or falling edge trigger (FTSR)
    EXTI->RTSR |= 0x00001;   // Enable rising edge trigger on EXTI0

    EXTI->FTSR |= 0x00001;   // Enable falling edge trigger on EXTI0

    // Mask the used external interrupt numbers.
    EXTI->IMR |= 0x00001;    // Mask EXTI0

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI0_IRQn, 1); // Priority level 1

    // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI0_IRQn);

    while(1)
    {
        // Do nothing.
    }

    return 0;
}
