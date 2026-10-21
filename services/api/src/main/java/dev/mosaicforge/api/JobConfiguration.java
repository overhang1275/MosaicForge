package dev.mosaicforge.api;

import java.time.Clock;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.env.Environment;

@Configuration
@ConditionalOnProperty(name = "mosaicforge.jobs.enabled", havingValue = "true")
class JobConfiguration {
  @Bean
  JobStore jobStore(Environment env, ProjectStore projects) {
    return new JdbcJobStore(
        env.getRequiredProperty("mosaicforge.persistence.url"),
        env.getRequiredProperty("mosaicforge.persistence.user"),
        env.getRequiredProperty("mosaicforge.persistence.password"),
        Clock.systemUTC(),
        5);
  }

  @Bean(destroyMethod = "close")
  WorkNotifications workNotifications(Environment env) {
    return new LettuceWorkNotifications(
        env.getRequiredProperty("mosaicforge.jobs.valkey-uri"),
        "mosaicforge:jobs",
        env.getRequiredProperty("mosaicforge.jobs.valkey-password"));
  }

  @Bean(initMethod = "start", destroyMethod = "close")
  JobRunner jobRunner(JobStore store, WorkNotifications notifications, EnginePort engine) {
    return new JobRunner(store, notifications, engine);
  }
}
